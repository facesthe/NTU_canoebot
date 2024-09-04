//! Silence laser crab

use std::{
    collections::hash_map::DefaultHasher,
    error::Error,
    hash::{Hash, Hasher},
    io::Cursor,
    str::FromStr,
    time::Duration,
};

use async_once::AsyncOnce;
use async_trait::async_trait;
use image::{EncodableLayout, RgbaImage};
use lazy_static::lazy_static;
use ntu_canoebot_util::debug_println;
use teloxide::{
    prelude::*,
    types::{InputFile, Me},
};

use ntu_canoebot_config as config;
use text_to_png::TextRenderer;

use self::virt_fs_cache::VIRT_FS;

use super::HandleCommand;

lazy_static! {
    // / A PNG image representation of the laser crab.
    static ref SILENCE_CRAB: AsyncOnce<RgbaImage> = AsyncOnce::new(async {
        let resp = reqwest::get(config::MISC_SILENCE_CRAB_URL)
            .await
            .expect("failed to fetch crab template image");

        let image_bytes = resp.bytes()
            .await
            .expect("failed to get the underlying response bytes");

        let img =
        image::load_from_memory_with_format(image_bytes.as_bytes(), image::ImageFormat::Png)
            .expect("failed to read png image")
            .into_rgba8();

        img
    });
}

#[derive(Clone, Debug)]
pub struct Silence {
    text: Option<String>,
}

impl FromStr for Silence {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            text: match input.len() {
                0 => None,
                _ => Some(input.to_owned()),
            },
        })
    }
}

#[async_trait]
impl HandleCommand for Silence {
    async fn handle_command(
        &self,
        bot: Bot,
        msg: Message,
        _me: Me,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut user = match msg.reply_to_message().and_then(|m| m.from.as_ref()) {
            Some(u) => u,
            None => {
                if let Some(t) = &self.text {
                    let silence_img = silence(t, None).await;
                    let dest = image_to_png_bytes(silence_img);

                    bot.send_photo(msg.chat.id, InputFile::memory(dest)).await?;
                }

                return Ok(());
            }
        };

        let mut silence_text = match &self.text {
            Some(t) => t.to_owned(),
            None => user.full_name(),
        };

        if user.id == _me.id {
            let chosen_noun = {
                let t = msg.date;
                let mut def_hash = DefaultHasher::new();
                t.hash(&mut def_hash);
                let val = def_hash.finish();

                let idx = val as usize % config::MISC_SILENCE_OFFENSIVE_NAMES.len();
                config::MISC_SILENCE_OFFENSIVE_NAMES[idx]
            };

            bot.send_message(
                msg.chat.id,
                format!("my spell doesn't work against me, you {}", chosen_noun),
            )
            .await?;

            // change the target to the person that sent the message
            user = msg.from.as_ref().unwrap();
            silence_text = chosen_noun.to_owned();
        }

        let img = virt_fs_cache::get_user_image_fs(bot.clone(), user.id).await?;

        let silence_img = silence(&silence_text, img).await;

        let dest = image_to_png_bytes(silence_img);
        bot.send_photo(msg.chat.id, InputFile::memory(dest)).await?;

        Ok(())
    }
}

#[allow(unused)]
pub async fn cleanup_fs_cache() -> Result<(), ()> {
    loop {
        use futures::stream::Collect;
        use futures::stream::StreamExt;

        let x = VIRT_FS.read_dir().await.unwrap().collect::<Vec<_>>().await;

        for path in x {
            let meta = path.metadata().await.unwrap();
        }

        tokio::time::sleep(Duration::from_secs(
            config::MISC_SILENCE_FS_CACHE_LIFETIME as u64 * 60,
        ))
        .await;
    }

    Ok(())
}

/// SILENCE something, or someone. Outputs a png image
async fn silence(target: &str, profile_pic: Option<RgbaImage>) -> RgbaImage {
    let bottom = SILENCE_CRAB.get().await;

    let starting_pos = {
        let x = (bottom.width() as f64 * config::MISC_SILENCE_TEXT_X_FRAC) as u32;
        let y = (bottom.height() as f64 * config::MISC_SILENCE_TEXT_Y_FRAC) as u32;

        (y, x)
    };

    // we do not want our target to spill over the template image, so
    // the font size is altered dynamically.
    let font_size = {
        const DEFAULT_FONT_SIZE: usize = 60;

        let pixels_available = (SILENCE_CRAB.get().await.width() as f64
            * (1_f64 - config::MISC_SILENCE_TEXT_X_FRAC)) as usize;

        let occupied_pixels = DEFAULT_FONT_SIZE * 2 / 3 * target.len();

        debug_println!("pixels available: {}", pixels_available);
        debug_println!("pixels occupied: {}", occupied_pixels);

        if occupied_pixels > pixels_available {
            let x = ((pixels_available / target.len()) as f32 * 1.75) as usize;
            debug_println!("new font size: {}", x);
            x
        } else {
            DEFAULT_FONT_SIZE
        }
    };

    let top = create_png_text(target, font_size);
    let top = image::load_from_memory_with_format(&top, image::ImageFormat::Png)
        .unwrap()
        .to_rgba8();

    debug_println!(
        "text image width: {}, height: {}",
        top.width(),
        top.height()
    );

    let mut res = overlay_images(bottom, &top, starting_pos).unwrap();
    if let Some(profile_p) = profile_pic {
        res = place_profile_picture(&profile_p, &res).unwrap();
    }

    res
}

fn create_png_text(text: &str, font_size: usize) -> Vec<u8> {
    let renderer = TextRenderer::new();

    let text_png = renderer
        .render_text_to_png_data(text, font_size, "#FFFFFF")
        .unwrap();

    text_png.data
}

/// Places the profile picture in the path of the crab's laser beams
fn place_profile_picture(profile_pic: &RgbaImage, crab: &RgbaImage) -> Result<RgbaImage, ()> {
    debug_println!("placing profile pic");

    let new_dims = (config::MISC_SILENCE_PIC_WIDTH_FRAC * crab.width() as f64) as u32;
    let profile_pic_new = image::imageops::resize(
        profile_pic,
        new_dims,
        new_dims,
        image::imageops::FilterType::Gaussian,
    );

    let top_left = {
        let top_col_frac =
            config::MISC_SILENCE_PIC_COL_FRAC - config::MISC_SILENCE_PIC_WIDTH_FRAC / 2.0;
        let left_row_frac =
            config::MISC_SILENCE_PIC_ROW_FRAC - config::MISC_SILENCE_PIC_WIDTH_FRAC / 2.0;

        (
            (left_row_frac * crab.height() as f64) as u32,
            (top_col_frac * crab.width() as f64) as u32,
        )
    };

    debug_println!("crab has width {}, height {}", crab.width(), crab.height());
    debug_println!("placing at coordinates: {:?}", top_left);

    overlay_images(crab, &profile_pic_new, top_left)
}

/// Overlay one `imagebuffer` on top of another,
/// at a given location (top left corner, as row-column coordinates) of the bottom image
fn overlay_images(
    bottom: &RgbaImage,
    top: &RgbaImage,
    location: (u32, u32),
) -> Result<RgbaImage, ()> {
    let mut bottom = bottom.clone();

    let (top_w, top_h) = (top.width(), top.height());
    let (bottom_w, bottom_h) = (bottom.width(), bottom.height());

    // overlay image needs to be smaller
    match (top_w < bottom_w, top_h < bottom_h) {
        (true, true) => (),
        _ => return Err(()),
    }

    let mut top_iter = top.pixels();
    for c in location.0..(location.0 + top_h) {
        for r in location.1..(location.1 + top_w) {
            match top_iter.next() {
                Some(p) => {
                    let x = p.0;
                    if x[3] != 0 {
                        // overwrite if alpha channel is not fully transparent
                        bottom[(r, c)] = p.to_owned()
                    }
                }
                None => break,
            }
        }
    }

    Ok(bottom)
}

/// Converts an image to it's actual byte level representation on disk.
fn image_to_png_bytes(img: RgbaImage) -> Vec<u8> {
    let mut dest = Cursor::new(Vec::<u8>::new());

    image::write_buffer_with_format(
        &mut dest,
        &img.to_vec(),
        img.width(),
        img.height(),
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .unwrap();

    dest.into_inner()
}

/// Cache for frequently used images downloaded from telegram
mod virt_fs_cache {
    use async_std::io::ReadExt;
    use image::RgbaImage;
    use ntu_canoebot_util::debug_println;

    use std::error::Error;

    use lazy_static::lazy_static;
    use teloxide::{net::Download, prelude::*};
    use tokio_util::compat::FuturesAsyncWriteCompatExt;
    use vfs::async_vfs::{AsyncMemoryFS, AsyncPhysicalFS, AsyncVfsPath};

    lazy_static! {
        /// Virtual in-memory filesystem
        pub static ref VIRT_FS: AsyncVfsPath = {
            if cfg!(debug_assertions) {
                AsyncPhysicalFS::new(std::env::current_dir().unwrap()).into()
            } else {
                AsyncMemoryFS::default().into()
            }
        };
    }

    /// Returns the first profile photo of a user.
    /// The photo is cached in a virtual filesystem (in memory/physical),
    /// depending on the config.
    ///
    /// If a user does not have a profile picture, returns None.
    pub async fn get_user_image_fs(
        bot: Bot,
        user: UserId,
    ) -> Result<Option<RgbaImage>, Box<dyn Error + Send + Sync>> {
        // look for file in filesystem first
        let filepath = VIRT_FS.join(format!("{}", user.0))?;

        // download file if it's not on disk
        if !filepath.exists().await? {
            let pics = bot.get_user_profile_photos(user).await?;
            if pics.photos.len() == 0 {
                return Ok(None);
            }

            let pic = pics.photos.first().unwrap().first().unwrap();
            let file = bot.get_file(&pic.file.id).await?;
            debug_println!("downloading image");
            download_file_from_user(bot, &file.path, user).await?;
        }

        let mut handle = filepath.open_file().await?;
        let mut buf = Vec::new();

        let _res = handle.read_to_end(&mut buf).await?;
        debug_println!("bytes read: {}", _res);

        let img = image::load_from_memory(&buf)?;
        debug_println!("image dims: {} by {}", img.width(), img.height());

        Ok(Some(img.to_rgba8()))
    }

    /// Downloads an image into the underlying virtual filesystem.
    /// The image is saved with the userid as a file name.
    pub async fn download_file_from_user(
        bot: Bot,
        path: &str,
        user: UserId,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let file = VIRT_FS.join(format!("{}", user.0))?.create_file().await?;

        // futures and tokio have very similar but incompatible traits
        let mut write_file = file.compat_write();

        bot.download_file(path, &mut write_file).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use image::EncodableLayout;
    use ntu_canoebot_config as config;
    use vfs::{async_vfs::AsyncMemoryFS, async_vfs::AsyncVfsPath};

    #[tokio::test]
    async fn test_in_memory_fs() {
        // let root: VfsPath = vfs::async_vfs::MemoryFS::new().into();
        let root: AsyncVfsPath = AsyncMemoryFS::new().into();
        // root.join("virtfile.txt").unwrap().create_file();

        root.join("virtfile.txt")
            .unwrap()
            .open_file()
            .await
            .unwrap();

        // println!("{:?}", root);

        // println!("{:?}", filesys);
        // let vfile = filesys.create_file("virtfile.txt").unwrap();
    }

    #[tokio::test]
    async fn test_silence() {
        // let silence_out = silence("ooooooooooooo", None).await;

        for size in 1..30 {
            let text = "o".repeat(size);

            let res = silence(&text, None).await;

            image::save_buffer(
                format!("silence_output-{}.png", size),
                &res,
                res.width(),
                res.height(),
                image::ColorType::Rgba8,
            )
            .unwrap();
        }
    }

    #[tokio::test]
    async fn test_superimpose_images() {
        let bottom = SILENCE_CRAB.get().await;

        // image::save_buffer_with_format(
        //     "silence_crab_processed.png",
        //     x.as_bytes(),
        //     x.width(),
        //     x.height(),
        //     image::ColorType::Rgba8,
        //     image::ImageFormat::Png,
        // )
        // .unwrap();

        let top = create_png_text("lalalalala", 60);

        let overlayed = overlay_images(
            &bottom,
            image::load_from_memory(&top).unwrap().as_rgba8().unwrap(),
            (20, 100),
        )
        .unwrap();

        image::save_buffer_with_format(
            "overlayed.png",
            &overlayed.as_bytes(),
            overlayed.width(),
            overlayed.height(),
            image::ColorType::Rgba8,
            image::ImageFormat::Png,
        )
        .unwrap();

        // println!("{:?}", x);
    }

    #[tokio::test]
    async fn test_superimpose_profile_pic() {
        // let user_profile_pic = get_user_image_fs(bot, user)
    }

    #[tokio::test]
    async fn test_get_template_from_url() {
        let resp = reqwest::get(config::MISC_SILENCE_CRAB_URL).await;

        let resp = if let Ok(r) = resp {
            r
        } else {
            panic!("invalid url");
        };

        std::fs::write("silence_crab_tempate.png", resp.bytes().await.unwrap()).unwrap();
    }

    #[test]
    fn test_render_png_text() {
        let text = "This is some text. Hello world!";

        let data = create_png_text(text, 40);

        std::fs::write("png_text_test.png", data).unwrap();
    }
}
