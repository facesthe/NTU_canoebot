import telebot

from canoebot_common_core import canoebot as bot

@bot.message_handler(commands=['start'])
def handle_start(message:telebot.types.Message):
    bot.send_message(
        message.chat.id,
        f'Hi {message.from_user.first_name}, this is NTU canoebot! '\
        'Browse the command list or type /help for more detailed instructions.'
    )
    return
