#https://docs.gspread.org/en/v3.7.0/api.html
import gspread
from oauth2client.service_account import ServiceAccountCredentials


class GoogleSheetClient:
  
  def __init__(self, key, sheet):
    scope = ['https://spreadsheets.google.com/feeds','https://www.googleapis.com/auth/drive']

    # add credentials to the account
    creds = ServiceAccountCredentials.from_json_keyfile_name('mymedia-359312-797dfbd3ff60.json', scope)
    self.client = gspread.authorize(creds)
    self.sheet = self.client.open_by_key(key)
    self.sheet_instance = self.sheet.get_worksheet(index = sheet)

  def insert_row(self, data, row):
    self.sheet_instance.insert_row(data, row)
  
  def insert_rows(self, datas, row):
    self.sheet_instance.insert_rows(datas, row)

  def append_row(self, data):
    self.sheet_instance.append_row(data)

    

#sheet = GoogleSheetClient(key = '1qlCRJ5wuAf7q5J37gwJa0MvMx0Vpgrm3lqzAD_VwyE4', sheetIndex = 0)
#sheet.insert_row(['1', '2', '3'], 2)
