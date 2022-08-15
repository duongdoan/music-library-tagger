import lib.utils as utils
import os
import sys
import music_tag

if len(sys.argv) != 2:
    print("Usage: {} PATH_TO_DIRECTORY".format(sys.argv[0]))
    root = "/Volumes/Music/Music1/V-Bolero/Best Compilation"
else:
    root = sys.argv[1]

def setTags(dir, album, albumArtist, genre):
    print('Preparing')
    totalFile = utils.countFiles(dir)
    print('Total number of files: ' + str(totalFile))
    countProgress = 0

    for dirname, dirs, files in os.walk(dir):
        for filename in files:
            try:
                fullPath = os.path.join(dirname, filename)
                f = music_tag.load_file(fullPath)
                f['album'] = album
                f['albumartist'] = albumArtist
                f['genre'] = genre
                f.save()
                print(fullPath)  
            except:
                pass

            countProgress = countProgress + 1
            print('Processed ' + str(countProgress) + '/' + str(totalFile) )
      
    print('Finished setting tags')


dir = "/Volumes/Music/Music1/V-Bolero/Best Compilation 4"
album = "Bolero Tuyển Chọn IV"
albumArtist = "Various Artists"
genre = "Nhạc vàng"            
setTags(dir, album, albumArtist, genre)
print('DONE')