import requests as rq
import json as jsn

import modules.settings as s

updates_url = f"https://api.telegram.org/bot{s.json.canoebot.apikey}/getUpdates"

send_msg_url = f"https://api.telegram.org/bot{s.json.canoebot.apikey}/sendMessage"

print(f"using config file {s._path}")


get_params = {
    "offset": None
}
response:rq.Response = rq.get(updates_url)

empty_post_response:rq.Response = rq.post(send_msg_url, {})

response_json = jsn.loads(response.text)
print(jsn.dumps(response_json, indent=4))
print("Post response:")
print(empty_post_response)
