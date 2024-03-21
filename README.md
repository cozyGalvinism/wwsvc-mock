# wwsvc-mock

[![codecov](https://codecov.io/gh/cozyGalvinism/wwsvc-mock/graph/badge.svg?token=1K75LAQCDT)](https://codecov.io/gh/cozyGalvinism/wwsvc-mock)

This crate aims to provide a mock server for the SoftENGINE ERP Suite webservices.

It is a dual purpose crate, providing both a library and a binary. The library exposes the router, which can be used to integrate it into an `axum-test` server
or a custom axum server. The binary is a standalone server.

Both the library and the binary can be extensively configured using environment variables or a configuration file.

## Installation

### As a library

```sh
cargo add wwsvc-mock
```

### As a binary

```sh
cargo install wwsvc-mock
```

## Usage

### Using environment variables

Configuring the server using environment variables is very cumbersome, but possible.

Here is an example .env file:

```env
APP__SERVER__BIND_ADDRESS=0.0.0.0:3000
APP__MOCK_RESOURCES=[{data_source={type=Empty},function=ARTIKEL,method=INSERT,revision=1,parameters={ARTNR=MeinArtikel}}]
```

### Using a configuration file

The same configuration can be expressed in a configuration file:

```toml
[app.server]
bind_address = "0.0.0.0:3000"

[[app.mock.resources]]
data_source.type = "Empty"
function = "ARTIKEL"
method = "INSERT"
revision = 1
parameters.ARTNR = "MeinArtikel"
```

Both ways of configuring the server can be combined. The environment variables will take precedence over the configuration file.
In this case, the mock server would allow you to register and deregister your service pass and additionally call the `ARTIKEL.INSERT` function with the given parameters. Using any other function, method or parameters will result in a 404 response in style of the original SoftENGINE ERP Suite webservice response.

You will also notice that we didn't specify any credentials, such as the vendor or app hash. This will cause the server to generate a random set of credentials for you. If you run the server using the binary, you will see the generated credentials in the logs. If you use the library, generating a config struct will also return the generated credentials.

### Mocking data sources

The mock server as a whole doesn't mock any data sources. Instead, you will need to provide which combinations of function, method and parameters you want to mock, along with the data the endpoint should return. Ideally, you return the "best case" scenario, but you can also return error responses.

Currently it is not possible to jitter response times or return random errors. This is a planned feature.

## Limitations

At this time, some limitations apply:

* As stated above, it is not possible to jitter response times or return random errors.
* The server does not support file uploads of any kind.
  * This is a planned feature.
* The server currently does not validate which HTTP verb you use, although it shouldn't need to.
