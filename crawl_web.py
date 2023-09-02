import requests
import re
import time
from lib.gsheet_client import GoogleSheetWriter


url = 'https://classicalpippo9.blogspot.com/search/label/Boxset'

googleSheet = GoogleSheetWriter(key = '1qlCRJ5wuAf7q5J37gwJa0MvMx0Vpgrm3lqzAD_VwyE4', sheetName = url.replace(':',''))
insertRowIndex = 1
rowsToAdd = [['Name', 'Link']]

crawUrl = url
depth = 0
while crawUrl != '' and depth < 1000:
  print(crawUrl)
  try: 
    r = requests.get(crawUrl)
    html = r.text.replace('\n',' ')
    posts = re.findall(r"\<h3.*?h3\>",html)
    for post in posts:
      titleLink = re.findall(r"\<a.*?a\>",post)[0]
      link = re.findall(r"(?<=href=').*?(?=')",titleLink)[0]
      name = re.findall(r"(?<=>).*?(?=<)",titleLink)[0]
      rowsToAdd.append([name, link])

      if (len(rowsToAdd) == 100):
        googleSheet.insert_rows(rowsToAdd, insertRowIndex)
        insertRowIndex = insertRowIndex + 100
        rowsToAdd = []

    olderPostBlock = re.findall(r"\<span id='blog-pager-older-link'.*?span\>",html)
    if (len(olderPostBlock) > 0):
      olderPostBlock = olderPostBlock[0]
      olderPostLinkBlock = re.findall(r"\<a.*?a\>",olderPostBlock)[0]
      #print(olderPostLinkBlock)
      olderPostLink = re.findall(r"(?<=href=').*?(?=')",olderPostLinkBlock)
      crawUrl = olderPostLink[0]
    else:
      crawUrl = ''
      print ('No more posts')
    depth = depth + 1
    time.sleep(5)
  except Exception as e:
    time.sleep(1)
    print(e)
    pass

if (len(rowsToAdd) > 0):
  googleSheet.insert_rows(rowsToAdd, insertRowIndex)
