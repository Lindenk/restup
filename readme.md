[![Build](https://github.com/Lindenk/restup/actions/workflows/build.yml/badge.svg)](https://github.com/Lindenk/restup/actions/workflows/build.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
![AUR](https://img.shields.io/aur/version/restup-bin)

# Restup

Restup is a tool for quickly and simply deploying a ReST server using local files. The intention of this project is to allow [Home Assistant](https://www.home-assistant.io/) to easily integrate with a system through it's [Restful](https://www.home-assistant.io/integrations/rest/) integrations.

## Usage

On it's own the server only provides it's status on `/` by returning `on` on a `GET`. A `sensors` directory can be specified with the `-s` flag, which will serve it's contents on `/sensors/raw/<file_path>`. A `commands` directory can also be specified, using the `-d` flag. This will allow running executables within the directory by `POST`ing to `/commands/raw/<file_path>`.

### Sensor Data

In addition to the raw endpoints, the entire sensor directory file contents can be serialized into json using the `/sensors` endpoint.

## Example

Launch the server with:

```bash
mkdir sensors commands
echo hello > sensors/world
restup -s ./sensors -d ./commands -p 3001 -i 127.0.0.1
```

And query it with:

```bash
curl localhost:3001/sensors/raw/hello # returns a 200 response with 'world'
curl localhost:3001/sensors # returns a 200 response with '{ "hello": "world" }'
```
