import music_tag

f = music_tag.load_file("/Volumes/Music/Music1/V-Bolero/Best Compilation/01 Truong Vu - 16 Trang Tron.flac")

# dict access returns a MetadataItem
title_item = f['artist']

# MetadataItems keep track of multi-valued keys
title_item.values  # -> ['440Hz']

# A single value can be extracted
title_item.first  # -> '440Hz'
title_item.value  # -> '440Hz'

# MetadataItems can also be cast to a string
print(str(title_item))  # -> '440Hz'