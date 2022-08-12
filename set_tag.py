import os
import sys
import music_tag

if len(sys.argv) != 2:
    print("Usage: {} PATH_TO_DIRECTORY".format(sys.argv[0]))
    root = "/Volumes/Music/Music1/V-Bolero/Best Compilation"
else:
    root = sys.argv[1]

def setTags(dir, album, albumArtist, genre):
    
    for dirname, dirs, files in os.walk(dir):
        #print(dirname)     # relative path (from cwd) to the directory being processed
        #print(dirs)       # list of subdirectories in the currently processed directory
        #print(files)       # list of files in the currently processed directory

        for filename in files:
            try:
              f = music_tag.load_file(os.path.join(dirname, filename))
              f['album'] = album
              f['albumartist'] = albumArtist
              f['genre'] = genre
              f.save()
              print(f)  
            except:
              pass
            
setTags("/Volumes/Music/Music1/V-Bolero/Best Compilation", "Bolero Tuyển Chọn I", "Various", "Nhạc vàng")
print('DONE')