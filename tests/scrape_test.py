import requests
import dotenv

dotenv.load_dotenv()
api_key = dotenv.get_key(dotenv.find_dotenv(), "API_SECRET")
client_id = dotenv.get_key(dotenv.find_dotenv(), "USER_ID")
api_base = dotenv.get_key(dotenv.find_dotenv(), "API_BASE")

ranked_api = api_base + "/api/v2/beatmapsets/search?m=3&q=key%3D4&s=ranked&nsfw="
loved_api = api_base + "/api/v2/beatmapsets/search?m=3&q=key%3D4&s=loved&nsfw="
qualified_api = api_base + "/api/v2/beatmapsets/search?m=3&q=key%3D4&s=qualified&nsfw="

def get_beatmapsets(api, token):
    cursor_string = ""
    counter = 0
    while True:
        r = requests.get(api + cursor_string, headers={"Authorization": "Bearer " + token})
        if r.status_code != 200:
            print("Error: {}".format(r.status_code))
            return
        data = r.json()
        for beatmaps in data["beatmapsets"]:
            counter += 1
            print("{}: {}".format(counter, beatmaps["title"]))
        if data["cursor_string"] is None:
            break
        cursor_string = "&cursor_string={}".format(data["cursor_string"])
        print(counter)
    return counter

token = requests.post(api_base + "/oauth/token",
    data={"client_id": client_id,
        "client_secret": api_key,
        "grant_type":"client_credentials",
        "scope":"public"
    }
).json()["access_token"]

_ = get_beatmapsets(loved_api, token)