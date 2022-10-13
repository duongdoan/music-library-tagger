import os
import shutil
import music_tag
import lib.utils as utils
from lib.gsheet_client import GoogleSheetReader

header = ["File", "Album", "Album Artist", "Title",
          "Artist", "Composer", "Genre", "Directory", "Album OK", "Album Artist OK", "Title OK", "Artist OK"]

def update(googleSheetKey, sheetCurrent, sheetUpdate, updateTag, moveFiles) :
    googleSheetOriginal = GoogleSheetReader(key = googleSheetKey, sheetName = sheetCurrent)
    originals = googleSheetOriginal.read_all()

    googleSheetNew = GoogleSheetReader(key = googleSheetKey, sheetName = sheetUpdate)
    news = googleSheetNew.read_all()

    changes = []

    for idx, meta in enumerate(originals):
        if (news[idx]["Album"] != "" and originals[idx]["File"] == news[idx]["File"] and (originals[idx]["Album"] != news[idx]["Album"] or originals[idx]["Album Artist"] != news[idx]["Album Artist"]
        or originals[idx]["Title"] != news[idx]["Title"] or originals[idx]["Artist"] != news[idx]["Artist"] or originals[idx]["Composer"] != news[idx]["Composer"])):
            changes.append(news[idx])

    print (len(originals))
    print (len(news))
    print (len(changes))

    if (updateTag):
        countReplaced = 0
        skip = 0
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
                    f.save()
                except:
                    print("Error update file")
                    print(fileMeta)
            countReplaced = countReplaced + 1
            if countReplaced%100 == 0:
                print (countReplaced)
            

        
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

dir = "/Volumes/Music"
current = "/Volumes/DD2/Classified"
changed = "/Volumes/DD2/Classified-Fixed"
willUpdate = False
willMove = True

update(googleSheetKey='1qlCRJ5wuAf7q5J37gwJa0MvMx0Vpgrm3lqzAD_VwyE4', sheetCurrent=current, sheetUpdate=changed, updateTag = willUpdate, moveFiles = willMove)

print('DONE')
