from fshare import FSAPI

URL = 'https://www.fshare.vn/folder/ZH78AID2UUHM'

bot = FSAPI(email="duongndoan@yahoo.com.au", password="")
login = bot.login()
print(login)
#home = bot.get_home_folders()
#print(home)
folder = bot.get_folder_urls(URL)
print(folder)
#sillicon_valley_ss1 = bot.get_folder_urls(URL)

#for episode in sillicon_valley_ss1:
#    print(episode['name'], bot.download("https://www.fshare.vn/file/{}".format(episode['linkcode'])))