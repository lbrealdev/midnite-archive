@setup:
    pre-commit install

[working-directory: 'scripts']
@run *arg:
    ./channel_list_gen.sh {{ arg }}

[working-directory: 'scripts/video']
@rename *arg:
    ./special_rename.sh {{ arg }}