from lib.fshare_client import FSAPI
import json

f = open('config/fshare.json')
fshareConfig = json.load(f)

URL = 'https://www.fshare.vn/folder/CV6XVAKRRD7M?token=1660548247'

bot = FSAPI(email=fshareConfig['email'], password=fshareConfig['password'])
login = bot.login()
folder = bot.get_folder_urls(URL)
print(folder)
