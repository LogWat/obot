import requests
import dotenv
import time

dotenv.load_dotenv()
api_key = dotenv.get_key(dotenv.find_dotenv(), "API_SECRET")
client_id = dotenv.get_key(dotenv.find_dotenv(), "USER_ID")
api_base = dotenv.get_key(dotenv.find_dotenv(), "API_BASE")
download_base = dotenv.get_key(dotenv.find_dotenv(), "DOWNLOAD_BASE")

ranked_api = api_base + "/api/v2/beatmapsets/search?m=3&q=key%3D4&s=ranked&nsfw="
loved_api = api_base + "/api/v2/beatmapsets/search?m=3&q=key%3D4&s=loved&nsfw="
qualified_api = api_base + "/api/v2/beatmapsets/search?m=3&q=key%3D4&s=qualified&nsfw="
graveyard_api = api_base + "/api/v2/beatmapsets/search?m=3&q=key%3D4&s=graveyard&nsfw="

def get_beatmapsets(api, token):
    cursor_string = ""
    counter = 0
    while True:
        r = requests.get(api + cursor_string, headers={"Authorization": "Bearer " + token})
        if r.status_code != 200:
            print("Error: {}".format(r.status_code))
            return
        data = r.json()
        print(data)
        if data["cursor_string"] is None:
            break
        cursor_string = "&cursor_string={}".format(data["cursor_string"])
        time.sleep(5)
    return counter


def write_beatmapsets(api, token, filename):
    cursor_string = ""
    counter = 0
    with open(filename, "w") as f:
        while True:
            r = requests.get(api + cursor_string, headers={"Authorization": "Bearer " + token})
            if r.status_code != 200:
                print("Error: {}".format(r.status_code))
                return
            data = r.json()
            for beatmaps in data["beatmapsets"]:
                counter += 1
                f.write("{}: {}-{} (by {})\n".format(counter, beatmaps["id"], beatmaps["title"], beatmaps["artist"]))
            if data["cursor_string"] is None:
                break
            cursor_string = "&cursor_string={}".format(data["cursor_string"])
            time.sleep(5)
    return counter

token = requests.post(api_base + "/oauth/token",
    data={"client_id": client_id,
        "client_secret": api_key,
        "grant_type":"client_credentials",
        "scope":"public"
    }
).json()["access_token"]

def download(beatmapsets_id, filename):
    url = download_base + "{}?n=1".format(beatmapsets_id)
    headers = {"User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/80.0.3987.132 Safari/537.36"}
    res = requests.get(url, headers=headers, stream=True)
    if res.status_code == 200:
        with open(filename, "wb") as f:
            for chunk in res.iter_content(chunk_size=1024):
                if chunk:
                    f.write(chunk)
    print("Downloaded {}".format(filename))


_ = get_beatmapsets(ranked_api, token)
#_ = write_beatmapsets(graveyard_api, token, "graveyard.txt")