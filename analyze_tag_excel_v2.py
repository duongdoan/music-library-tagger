import os
import music_tag
import lib.utils as utils
from lib.excel_client import ExcelSheetWriter

header = ["File", "Album", "Album Artist", "Title", "Artist", "Composer", "Genre", "Compilation", "Comment", "Artwork", "Directory"]


def analyze(dir, path, prepare):
    totalFile = 0
    if prepare:
        print('Preparing')
        totalFile = utils.countFiles(dir)
        print('Total number of files: ' + str(totalFile))
    countProgress = 0
    print('Progress')
    sheet = ExcelSheetWriter(path = path, sheetName = dir.replace('/', '--'))
    sheet.insert_row(header, 1)
    row = 2
    rowsToAdd = []
    for dirname, dirs, files in os.walk(dir):
        for filename in files:
            try:
                fullPath = os.path.join(dirname, filename)
                f = music_tag.load_file(fullPath)
                data = [filename, str(f['album']), str(f['albumartist']), str(f['title']), str(f['artist']), str(f['composer']), str(f['genre']), str(f['compilation']), str(f['comment']), str(f['artwork']), dirname]
                rowsToAdd.append(data)
            except Exception as e:
                pass
            if (len(rowsToAdd) == 100):
                sheet.insert_rows(rowsToAdd, row)
                row = row + 100
                rowsToAdd = []
            countProgress = countProgress + 1
            print(' ' + str(countProgress), end='\r', flush=True)
            

    if (len(rowsToAdd) > 0):
        sheet.insert_rows(rowsToAdd, row)
    print('')
    print('Finished analyzing, processed ' + str(countProgress))


dir = "/Volumes/Music/Music4"
path = "analyze3.xlsx"

analyze(dir, path=path, prepare=False)

print('DONE')
