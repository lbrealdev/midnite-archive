# Troubleshooting

## yt-dlp

// to do

## vlc

If the video start with sound but black screen, check the codec:

- Codec: H264 - MPEG-4 AVC (part 10) (avc1)
- Codec: AOMedia's AV1 Video (av01)

Execute the video using `ffplay` from ffmpeg package to test it:
```shell
ffplay <video-file-path>
```

## Related links

- (VLC not working well --- Black screen (with sound) for videos, while other players work just fine)[https://superuser.com/a/1555446/1731833]
