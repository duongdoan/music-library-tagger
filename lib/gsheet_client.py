#https://docs.gspread.org/en/v3.7.0/api.html
import gspread
from oauth2client.service_account import ServiceAccountCredentials


class GoogleSheetWriter:
  
  def __init__(self, key, sheetName):
    scope = ['https://spreadsheets.google.com/feeds','https://www.googleapis.com/auth/drive']

    # add credentials to the account
    creds = ServiceAccountCredentials.from_json_keyfile_name('config/mymedia-359312-797dfbd3ff60.json', scope)
    self.client = gspread.authorize(creds)
    self.sheet = self.client.open_by_key(key)
    self.sheet_instance = self.sheet.add_worksheet(title=sheetName, rows = 0, cols=20)

  def insert_row(self, data, row):
    self.sheet_instance.insert_row(data, row)
  
  def insert_rows(self, datas, row):
    self.sheet_instance.insert_rows(datas, row)

  def append_row(self, data):
    self.sheet_instance.append_row(data)

    

class GoogleSheetReader:
  
  def __init__(self, key, sheetName):
    scope = ['https://spreadsheets.google.com/feeds','https://www.googleapis.com/auth/drive']

    # add credentials to the account
    creds = ServiceAccountCredentials.from_json_keyfile_name('config/mymedia-359312-797dfbd3ff60.json', scope)
    self.client = gspread.authorize(creds)
    self.sheet = self.client.open_by_key(key)
    self.sheet_instance = self.sheet.worksheet(title=sheetName)

  def read_all(self):
    return self.sheet_instance.get_all_records()

    
#sheet = GoogleSheetClient(key = '1qlCRJ5wuAf7q5J37gwJa0MvMx0Vpgrm3lqzAD_VwyE4', sheetIndex = 0)
#sheet.insert_row(['1', '2', '3'], 2)
