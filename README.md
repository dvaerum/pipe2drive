# pipe2drive
pipe2drive is a simple program I wrote for myself, because I wanted a way to create a tarball of my files and upload them to Google Drive without having to store the tarball on my system. I didn't want to store the tarball on my system because that would require around as much free space as the data itself, since it wasn't compressible.

## How the program works
You need a credential from Google before you can use pipe2drive. When created, you need to download the credentials and place them here `~/.config/pipe2drive/client_secret.json` or use the `--secret <FILE>` option to select a different location.

The first time using pipe2drive, you will have to create a token. This token is stored here `~/.config/pipe2drive/client_token.json` or you can pick a different location by using the option `--token <FILE>`

When using pipe to upload a file to Drive. One thing you have to do is selecting the size of the file you are uploading. Of course, you may not know the size since you are probably uploading data while it is being created. That is okay, you can use an estimate.

If the data you are uploading is bigger than you estimated, multiple files will be uploaded. The first file will be renamed to FILE_NAME.000, the next file will be named FILE_NAME.001 and so on until all the data is uploaded.

If the data is less than you estimated, the file will be uploaded and the difference between what you estimated and the actual size will be filled with the value NULLs (0x00). This has to happen in order to complete the upload.


## Help Menu
```
Pipe2Google 0.1.0
If you pipe data (doesn't matter what data) to this program and then select a name for that data and declare it size, it
will be uploaded to Google Drive

USAGE:
    pipe2drive [FLAGS] [OPTIONS] <size>

FLAGS:
        --duplicate    Allow multiple files to have the same name
    -h, --help         Prints help information
        --replace      If a file exists with the same name it will be replaced
    -V, --version      Prints version information

OPTIONS:
        --secret <FILE>       Select the file containing the client secret. If you don't have one go here
                              https://console.developers.google.com/apis/credentials
        --token <FILE>        Select the file/there the file containing the client token is/should be saved
    -n, --name <FILE NAME>    The name of the file uploaded to Google Drive
        --folder <ID>         The ID of the folder where you want the file to be uploaded to.
                              If this is not defined, the file will be uploaded to 'My Drive'

ARGS:
    <size>    The size of the data you want to upload.
              Example: 100mib, 1gb or 1048576 aka. 1mib)
              Supported Sizes: b, kb, kib, mb, mib, gb, gib, tb and tib
```
