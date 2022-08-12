import os
import sys
import music_tag

if len(sys.argv) != 2:
    print("Usage: {} PATH_TO_DIRECTORY".format(sys.argv[0]))
    root = "/Volumes/Music/Music1/V-Bolero/Best Compilation"
else:
    root = sys.argv[1]

ALBUM_NAME = "Bolero Tuyển Chọn I"
ALBUM_ARTIST = "Various"
GENRE = "Nhạc vàng"

for dirname, dirs, files in os.walk(root):
    print(dirname)     # relative path (from cwd) to the directory being processed
    #print(dirs)       # list of subdirectories in the currently processed directory
    #print(files)       # list of files in the currently processed directory

    for filename in files:
        try:
          f = music_tag.load_file(os.path.join(dirname, filename))
          del f['album artist']
          f['albumartist'] = ALBUM_ARTIST
          f['genre'] = GENRE
          f.save()
          print(f)  
        except:
          pass
        
        #print(os.path.join(dirname, filename))   # relative path to the "current" file

print('DONE')