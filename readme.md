# SHC

## share in minimum steps (Heavily Work in Progress)

## Install

```console
curl -fsSl https://shc.ajaysharma.dev/install.sh | sh
```

## How to use?

```console
Usage: shc [COMMAND]

Commands:
    login       login to use shc
    add         upload file
    list        list all files
    remove      remove file
    visibility  toggle file's visibility
    rename      rename file
    get         download file
    logout      logout from shc
    help        Print this message or the help of the given subcommand(s)

Options:
    -h, --help  Print help
```

### TODOs

- [ ] Share a portion of a file
- [ ] Resume Upload
- [ ] gracefull exit
- [ ] command aliases
- [ ] improve code by studying aim
- [ ] highlight imp words in output
- [ ] new text file
- [ ] custom download path
- [ ] set upload_status to failed if get some error
- [ ] shc get < link / id >
- [ ] can we render html on cli or backend?
- [ ] Path vs PathBuf
- [ ] make config_path part of AppConfig
- [ ] dynamic name width?
- [ ] install script -WIP
- [ ] pretty error messages
- [ ] generic config to create more config like user_config
- [ ] make user and userInfo same
- [ ] fix mut & if needed
- [ ] better email otp template
- [ ] update timeago interactively
- [ ] localize consts
- [ ] pre-compiled binaries
