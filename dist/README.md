SWISS KIT
=========

(Turkce: README.tr.md dosyasina bakin.)

Download YouTube and TikTok videos as MP4 or MP3, and convert images, audio
and video (WEBP to JPG, MOV to MP4, WAV to MP3 and more). Everything runs on
your own PC. Nothing is uploaded anywhere.


WHAT YOU NEED (one-time setup)
------------------------------

Swiss Kit uses three free tools to do the heavy lifting:

  - yt-dlp   downloads the videos
  - ffmpeg   makes the MP3s and converts files
  - deno     lets yt-dlp read YouTube pages

Easiest way to get them:

  1. Double-click "Install tools.bat" in this folder.
  2. Wait until it says "All done!". It sets everything up by itself
     (using winget or Scoop) and tries again if a download drops. If it
     reports a failure, just run it again.

If you only want to convert images (WEBP to JPG and so on), you can skip this
step. Image conversion needs none of these tools.

If the installer does not work
------------------------------
Open a normal PowerShell window (not "Run as administrator") and run these
lines one at a time:

    Set-ExecutionPolicy -Scope CurrentUser RemoteSigned
    irm get.scoop.sh | iex
    scoop install yt-dlp ffmpeg deno

If a download fails halfway (connection reset, "could not be resolved"), run
the same command again. If it keeps failing, try a different network or a VPN
for the install. You can switch it off afterwards.


HOW TO START
------------

  1. Extract the whole zip (right-click, Extract All). Do not run it from
     inside the zip.
  2. Double-click "Start Swiss Kit.bat".
  3. Your browser opens with Swiss Kit. Keep the black window open while you
     use it. Closing that window quits the app.

The first time, Windows may say "Windows protected your PC". That happens
because the app is new and not signed by a big company. Click "More info",
then "Run anyway". If you would rather not trust a file from a friend, you can
build it yourself from the source (see the end of this file).

Swiss Kit only listens on your own computer (127.0.0.1). Other devices and
websites cannot use it.


USING IT
--------

Download a video
  1. Pick a site on the left (see "Supported sites" below).
  2. Paste the link and click Fetch.
  3. Choose MP4 (video) or MP3 (audio only). For MP4 you can pick the quality.
  4. Click Download. Watch the progress in the queue.
  5. When it says Done, click "Show in folder".

Supported sites
  YouTube, TikTok, Instagram, X (Twitter), Facebook, Vimeo, Reddit,
  SoundCloud, Twitch, Dailymotion and Pinterest each have their own button.
  "Any site" accepts a link from almost any other video or audio site that
  yt-dlp understands. If a site is not supported, you will see an error.

  Good to know:
  - Only public posts work. Private posts, or posts that need you to be
    logged in (common on Instagram, Facebook and X), cannot be downloaded.
  - SoundCloud is audio only, so it always saves MP3.
  - Playlists and whole channels are not supported. Paste one link at a time.
  - Sites change often. If one stops working, update yt-dlp (see below).

Language
  Use the EN / TR switch at the bottom of the left bar. Swiss Kit remembers
  your choice. The first time, it picks Turkish if your browser is Turkish.

Convert a file
  1. Pick Images, Audio or Video on the left.
  2. Choose what to convert to (for example JPG).
  3. Drop your files on the box, or click it to choose files. You can pick
     many files at once.

Quick Convert (bottom right) jumps straight to the common jobs, like
WEBP to JPG or MP4 to MP3.

Where do my files go?
  Downloads\Swiss Kit  (inside your Downloads folder).
  The sidebar always shows the exact folder, and "Open folder" opens it.
  Swiss Kit never overwrites a file. If "photo.jpg" already exists, the new
  one is saved as "photo (1).jpg".

What can it convert?
  Images   WEBP, PNG, JPG, GIF, BMP, TIFF (to and from each other)
  Audio    MP3, WAV, M4A, OGG, FLAC
  Video    MP4, WEBM, GIF, or extract the audio as MP3
  Not yet  HEIC photos from iPhones


IF SOMETHING GOES WRONG
-----------------------

The sidebar shows a green dot next to yt-dlp and ffmpeg when they are found.

Red dot, or "Not found on this PC"
  Run "Install tools.bat". If you just installed them, restart your PC once so
  Windows picks them up, then start Swiss Kit again.

A download fails, or YouTube suddenly stops working
  YouTube changes often and yt-dlp must keep up. Update it, then try again:
      winget upgrade yt-dlp.yt-dlp
  or, if you used scoop:
      scoop update yt-dlp

"That link does not belong to ..."
  You are on the wrong site button. Pick the right site on the left, or use
  "Any site". Playlists are not supported. Paste a link to a single video.

A download says it failed on Instagram, Facebook or X
  Those sites often hide posts from people who are not logged in. Public
  posts usually work. Private ones never will.

The browser did not open
  Look at the black window. It prints the address, usually
  http://127.0.0.1:7878. Type that into your browser.

Black window closes immediately
  Make sure you extracted the zip first and that "swiss-kit-app.exe" sits
  next to "Start Swiss Kit.bat".

Windows Defender or an antivirus blocks the file
  This can happen with any new unsigned program. If you trust where it came
  from, allow it, or build it yourself from the source.


FAIR USE
--------

Only download things you own, that are free to use, or that you have
permission to save. Respect the rules of each website and the rights of the
people who made the video.


BUILDING FROM SOURCE (for the curious)
--------------------------------------

You need Rust (https://rustup.rs). On Windows, Rust also needs C++ build
tools; the GNU toolchain with MinGW works too (scoop install mingw).

  1. Open the source folder in a terminal.
  2. Double-click "build-release.bat", or run:
         cargo build --release -p swiss-kit-app
  3. The program is target\release\swiss-kit-app.exe.

Command-line version (no browser). "get" works out the site by itself:

    cargo run -p swiss-kit -- get "LINK" -f mp3 -o "%USERPROFILE%\Downloads"
    cargo run -p swiss-kit -- image photo.webp jpg
    cargo run -p swiss-kit -- media clip.mov mp4

Options for the app:  swiss-kit-app.exe --out "D:\My Folder"   (choose the
save folder)   and   --no-open   (do not open the browser automatically).

Mac and Linux: the app builds with the same cargo command and uses the same
tools (brew install yt-dlp ffmpeg deno). Mac has not been tested.
