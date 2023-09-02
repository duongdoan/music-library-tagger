import os
from send2trash import send2trash

def run(dir):
    totalFile = 0
    
    countProgress = 0
    print('Analyzing')
    
    for dirname, dirs, files in os.walk(dir):
        for filename in files:
            try:
                filePath = os.path.join(dirname, filename)
                  

                if (filePath.endswith('.cue')):
                  baseFile = filePath.replace('.cue', '')
                  flacFile = baseFile + '.flac'
                  apeFile = baseFile + '.ape'
                  print(filePath + '\r')
                  #send2trash(filePath)
                  send2trash(flacFile)
                  send2trash(apeFile)
                  
                if (filePath.endswith('ape')):
                  print(filePath + '\r')
                  send2trash(filePath)

            except Exception as e:
                print(e)
            
            countProgress = countProgress + 1
            print(' ' + str(countProgress) + '/' + str(totalFile), end='\r', flush=True)
            

    
    print('Finished analyzing, processed ' + str(countProgress))


dir = "/Users/duongdoan/Documents/Music/Mercury Living Presence Vol. 2"


run(dir)

print('DONE')
