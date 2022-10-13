import os
import music_tag
import lib.utils as utils
from lib.gsheet_client import GoogleSheetWriter

header = ["File", "Album", "Album Artist", "Title",
          "Artist", "Composer", "Genre", "Directory", "Album OK", "Album Artist OK", "Title OK", "Artist OK"]

def analyze(dir, googleSheetKey, prepare):
    totalFile = 0
    if prepare:
        print('Preparing')
        totalFile = utils.countFiles(dir)
        print('Total number of files: ' + str(totalFile))
    countProgress = 0

    googleSheet = GoogleSheetWriter(key = googleSheetKey, sheetName = dir)
    googleSheet.insert_row(header, 1)
    row = 2
    rowsToAdd = []
    for dirname, dirs, files in os.walk(dir):
        for filename in files:
            try:
                fullPath = os.path.join(dirname, filename)
                f = music_tag.load_file(fullPath)
                data = [filename, str(f['album']), str(f['albumartist']), str(f['title']), str(f['artist']), str(f['composer']), str(f['genre']), dirname]
                rowsToAdd.append(data)
            except Exception as e:
                pass
            if (len(rowsToAdd) == 100):
                googleSheet.insert_rows(rowsToAdd, row)
                row = row + 100
                rowsToAdd = []
            countProgress = countProgress + 1
            if (countProgress % 100 == 0):
                print('Processed ' + str(countProgress) + '/' + str(totalFile))

    if (len(rowsToAdd) > 0):
        googleSheet.insert_rows(rowsToAdd, row)
    print('Finished analyzing, processed ' + str(countProgress))


dir = '/Volumes/DD2/Classified'


analyze(dir, googleSheetKey='1qlCRJ5wuAf7q5J37gwJa0MvMx0Vpgrm3lqzAD_VwyE4', prepare=False)

print('DONE')
