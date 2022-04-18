# fbihtax

**WORK IN PROGRESS** - basic version available, used personally, but not thoroghly tested

Simple CLI tool to help manage tax payments in FBiH (Bosnia and Herzegovina Federation) written in Rust.

Currently PDF output requires `pdftk` to be installed, otherwise `fdf` or `json` outputs can be used.

## Installation

Check out releases for built binaries. Script is also available for installation, but it requires root access. Script will also install `pdftk` version required to properly run this tool.
```
$ sudo sh -c "$(curl -fsSL https://raw.githubusercontent.com/esensar/fbihtax/main/scripts/install.sh)"
```
or
```
$ sudo sh -c "$(wget -qO- https://raw.githubusercontent.com/esensar/fbihtax/main/scripts/install.sh)"
```
or
```
$ sudo sh -c "$(fetch -o - https://raw.githubusercontent.com/esensar/fbihtax/main/scripts/install.sh)"
```

## Usage

This is preferrably used automatically on some server, but it can also be used manually if needed. For automatic example check out [fbihtax-example-project](https://github.com/esensar/fbihtax-example-project) which can also be used as a template.

For more information use `fbihtax --help`.

### Generating AMS form

This tool supports multiple formats for AMS form (PDF being the most interesting probably). To generate simple PDF run:

```
$ fbihtax ams --income 1000.00 --output-format pdf
```
> Above command requires `pdftk` to be installed and available on path. If not available on path, path to the tool can be provided in configuration.

For more customization `fdf`, `xfdf` or `json` formats can be generated. To use custom fonts in the resulting PF:

```
$ fbihtax ams --income 1000.00 --output-format xfdf
$ pdftk amscache.pdf fill_form amsform.xfdf output amsform.pdf replacement_font "Your Font Family Here"
```
> Above command requires pdftk-java version above v3.3.0 since replacement_font was not available before

Check out `fbihtax ams --help` for more options.

### Generating tax breakdown

This tool can provide a basic tax breakdown to make it easier to calculate tax payments.

```
$ fbihtax tax-breakdown --income 1000.00
```

This will generate `taxbreakdown.json` in current directory. Check out `fbihtax tax-breakdown --help` for more options.

### Configuration

This tool looks for configuration in `.fbihtax.json` file in current working directory. Besides that main configuration, optionally separate user and client configuration JSON files can be provided.


For all options, check out [example configuration](examples/.fbihtax.json). [User](examples/user_info.json) and [client configuration](examples/client_config.json) JSON files are also present in [examples](examples) directory.

To provide user and client configurations, use `--user-config file_name.json` and `--client-config file_name.json`.

## License

[MIT](LICENSE)
