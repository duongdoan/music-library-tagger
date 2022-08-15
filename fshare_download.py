from fshare import FSAPI

URL = 'https://www.fshare.vn/folder/CV6XVAKRRD7M?token=1660548247'

bot = FSAPI(email="duongndoan@yahoo.com.au", password="1tr2s3v4b")
login = bot.login()
folder = bot.get_folder_urls(URL)
print(folder)
