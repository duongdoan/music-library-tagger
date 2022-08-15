import os
import music_tag
import lib.utils as utils
import xlsxwriter
from lib.gsheet_client import GoogleSheetClient

header = ["Directory", "File", "Album", "Album Artist", "Title",
          "Artist", "Album OK", "Album Artist OK", "Title OK", "Artist OK"]

def write_row(worksheet, row, data):
    for idx, x in enumerate(data):
        worksheet.write(row, idx, str(x))

def analyze(dir, out, prepare):
    totalFile = 0
    if prepare:
        print('Preparing')
        totalFile = utils.countFiles(dir)
        print('Total number of files: ' + str(totalFile))
    countProgress = 0
    workbook = xlsxwriter.Workbook(out)
    worksheet = workbook.add_worksheet()
    write_row(worksheet, 0, header)
    
    row = 1

    for dirname, dirs, files in os.walk(dir):
        for filename in files:
            try:
                fullPath = os.path.join(dirname, filename)
                f = music_tag.load_file(fullPath)
                data = [dirname, filename, str(f['album']), str(f['albumartist']), str(f['title']), str(f['artist'])]
                write_row(worksheet, row, data)
                row = row + 1
                #print(data)

            except:
                pass

            countProgress = countProgress + 1
            if (countProgress % 100 == 0):
                print('Processed ' + str(countProgress) + '/' + str(totalFile))
    workbook.close()
    print('Finished analyzing, processed ' + str(countProgress))


dir = "/Volumes/Temp"


analyze(dir, out='analyze.xlsx', prepare=True)

print('DONE')
