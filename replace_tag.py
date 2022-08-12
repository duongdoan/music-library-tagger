import os
import sys
import music_tag
import utils

if len(sys.argv) != 2:
    print("Usage: {} PATH_TO_DIRECTORY".format(sys.argv[0]))
    root = "/Volumes/Music/Music1/V-Bolero/Best Compilation"
else:
    root = sys.argv[1]

ALBUM_NAME = "Bolero Tuyển Chọn I"
ALBUM_ARTIST = "Various"
GENRE = "Nhạc vàng"

def fixAlbumTags(dir, albumName, albumArtist, genre):
    
    for dirname, dirs, files in os.walk(dir):
        #print(dirname)     # relative path (from cwd) to the directory being processed
        #print(dirs)       # list of subdirectories in the currently processed directory
        #print(files)       # list of files in the currently processed directory

        for filename in files:
            try:
              f = music_tag.load_file(os.path.join(dirname, filename))
              f['album'] = albumName
              f['albumartist'] = albumArtist
              f['genre'] = genre
              f.save()
              print(f)  
            except:
              pass
            

def checkAndReplaceTagName(dir, tag ,oldvalue, newValue, replace):
  print('Preparing')
  totalFile = utils.countFiles(dir)
  print('Total number of files: ' + str(totalFile))
  countProgress = 0
  countFound = 0
  countReplaced = 0

  for dirname, dirs, files in os.walk(dir):
        for filename in files:
            try:
              fullPath = os.path.join(dirname, filename)
              f = music_tag.load_file(fullPath)
              currentTag = str(f[tag])
              if currentTag.startswith(oldvalue):
                countFound = countFound + 1
                newTag = currentTag.replace(oldvalue, newValue)
                print(currentTag + '  ====>  ' + newTag)
                if replace:
                  f[tag] = newTag
                  f.save()
                  countReplaced = countReplaced + 1
            except:
              pass

            countProgress = countProgress + 1
            if (countProgress % 100 == 0):
              print('Processed ' + str(countProgress) + '/' + str(totalFile) )
  print ('Finished checking, found ' + str(countFound))
  print ('Finished replacing, replaced ' + str(countReplaced))


checkAndReplaceTagName('/Volumes/DD2/Music3/Classical/Violin', 'album', '16 / ', '', True)

#fixAlbumTags('/Volumes/DD2/Music2', "Bolero Tuyển Chọn I", "Various", "Nhạc vàng")
print('DONE')