import os


def run(dir):
    totalFile = 0
    
    countProgress = 0
    print('Analyzing')
    
    for dirname, dirs, files in os.walk(dir):
        for filename in files:
            try:
                filePath = os.path.join(dirname, filename)
                

                if (filePath.endswith('ape')):
                  print(filePath + '\r')

                  if (os.path.is_file(filePath)):
                    print(filePath + '\r')
                  #print(filePath + '\r')
                  #os.remove(filePath)

                if (filePath.endswith('.cue')):
                  baseFile = filePath.replace('.cue', '')
                  flacFile = baseFile + '.flac'
                  apeFile = baseFile + '.ape'
                  print(filePath + '\r')
                  os.remove(filePath)
                  if (os.path.is_file(flacFile)):
                    print(flacFile + '\r')
                    os.remove(flacFile)
                  if (os.path.is_file(apeFile)):
                    print(apeFile + '\r')
                    os.remove(apeFile)
            except Exception as e:
                pass
            
            countProgress = countProgress + 1
            print(' ' + str(countProgress) + '/' + str(totalFile), end='\r', flush=True)
            

    
    print('Finished analyzing, processed ' + str(countProgress))


dir = "/Users/duongdoan/Documents/Music"


run(dir)

print('DONE')
