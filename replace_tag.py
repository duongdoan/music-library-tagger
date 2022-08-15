import os
import sys
import music_tag
import lib.utils as utils

if len(sys.argv) != 2:
    print("Usage: {} PATH_TO_DIRECTORY".format(sys.argv[0]))
    root = "/Volumes/Music/Music1/V-Bolero/Best Compilation"
else:
    root = sys.argv[1]

def checkAndReplaceTagName(dir, tag, oldvalue, newValue, replace, prepare):
  totalFile = 0
  if prepare:
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
              print('Processed ' + str(countProgress) + '/' + str(totalFile) + ' - Replaced ' + str(countReplaced))
  print ('Finished checking, found ' + str(countFound))
  print ('Finished replacing, replaced ' + str(countReplaced))


dir = "/Volumes/Music/Music2"
tag = "album"
oldvalue = "16 / "
newValue = ""
doReplace = True

checkAndReplaceTagName(dir, tag, oldvalue, newValue, doReplace, prepare = False)

print('DONE')
