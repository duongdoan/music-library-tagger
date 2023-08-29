import os
import shutil
import music_tag
import lib.utils as utils
from lib.gsheet_client import GoogleSheetReader
from lib.gsheet_client import GoogleSheetWriter
import time

header = ["Dir", "File" ,"Album", "Album Artist", "Title", "Artist", "Composer", "Genre", "Compilation", "Comment", "Artwork", "Directory"]

def update(googleSheetKey, sheetCurrent, sheetUpdate, updateTag, moveFiles) :
    googleSheetOriginal = GoogleSheetReader(key = googleSheetKey, sheetName = sheetCurrent)
    originals = googleSheetOriginal.read_all()

    googleSheetNew = GoogleSheetReader(key = googleSheetKey, sheetName = sheetUpdate)
    news = googleSheetNew.read_all()

    googleSheetError = GoogleSheetWriter(key = googleSheetKey, sheetName = sheetCurrent + " Errors " + str(time.time()) )

    changes = []

    for idx, meta in enumerate(originals):
        if ( originals[idx]["File"] == news[idx]["File"] and (originals[idx]["Album"] != news[idx]["Album"] or originals[idx]["Album Artist"] != news[idx]["Album Artist"]
        or originals[idx]["Title"] != news[idx]["Title"] or originals[idx]["Artist"] != news[idx]["Artist"]  or 
        originals[idx]["Composer"] != news[idx]["Composer"] or originals[idx]["Genre"] != news[idx]["Genre"] or originals[idx]["Comment"] != news[idx]["Comment"])):
            changes.append(news[idx])

    print ('Preparing')
    print (len(originals))
    print (len(news))
    print (len(changes))
    print ('Replaced: ')
    if (updateTag):
        countReplaced = 0
        skip = 0
        errors = []
        errorIndex = 0
        for idx, fileMeta in enumerate(changes):
            if countReplaced >= skip:
                try:
                    fullPath = os.path.join(fileMeta["Directory"], fileMeta["File"])
                    f = music_tag.load_file(fullPath)
                    f['album'] = fileMeta["Album"]
                    f['albumartist'] = fileMeta["Album Artist"]
                    f['title'] = fileMeta["Title"]
                    f['artist'] = fileMeta["Artist"]
                    f['genre'] = fileMeta["Genre"]
                    f['composer'] = fileMeta["Composer"]
                    #f['compilation'] = fileMeta["Compilation"]
                    f['comment'] = fileMeta["Comment"]
                    f.save()
                except Exception as e:
                    errorIndex = errorIndex + 1
                    errors.append([fileMeta["File"], fileMeta["Album"], fileMeta["Album Artist"], fileMeta["Title"], fileMeta["Artist"], fileMeta["Composer"], fileMeta["Genre"], fileMeta["Directory"], str(e)])
                    

            countReplaced = countReplaced + 1
            print(' ' + str(countReplaced), end='\r', flush=True)
            
            
        if (len(errors) > 0):
            googleSheetError.insert_rows(errors, 1)

        print('')
        print (countReplaced)
    
    if (moveFiles):
        countMoved = 0
        folders = []
        newFolders = []
        for idx, fileMeta in enumerate(changes):
            if fileMeta["Directory"] not in folders:
                folder = fileMeta["Directory"]
                newFolder = folder.replace(current, current + "/Good")
                folders.append(folder)
                newFolders.append(newFolder)
                try:
                    shutil.move(folder, newFolder)
                    countMoved = countMoved + 1
                    print (folder)
                except:
                    print("Error---------------------------------------")
                    print(folder)
                if countMoved%10 == 0:
                    print (countMoved)
                
        

        print (countMoved)

current = "/Users/duongdoan/Documents/Music"
changed = "/Users/duongdoan/Documents/Music-Fixed"
willUpdate = True
willMove = False

update(googleSheetKey='1qlCRJ5wuAf7q5J37gwJa0MvMx0Vpgrm3lqzAD_VwyE4', sheetCurrent=current, sheetUpdate=changed, updateTag = willUpdate, moveFiles = willMove)

print('DONE')
