
import xlsxwriter

class ExcelSheetWriter:
  
  def __init__(self, path, sheetName):
    self.workbook = xlsxwriter.Workbook(path)
    self.worksheet = self.workbook.add_worksheet(sheetName)

  def insert_row(self, data, row):
    for idx, x in enumerate(data):
        self.worksheet.write(row, idx, str(x))
  
  def insert_rows(self, datas, row):
    for idx, data in enumerate(datas):
      self.insert_row(data, row + idx)

  def append_row(self, data):
    self.insert_row(data)

    

#class ExcelSheetReader:
  
  #def __init__(self, key, sheetName):
    # Do nothing

  #def read_all(self):
    # Do nothing

    
#sheet = GoogleSheetClient(key = '1qlCRJ5wuAf7q5J37gwJa0MvMx0Vpgrm3lqzAD_VwyE4', sheetIndex = 0)
#sheet.insert_row(['1', '2', '3'], 2)
