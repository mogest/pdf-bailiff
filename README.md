# pdf-bailiff

A web server that takes in a PDF and returns the text on that PDF.

Useful as a sidecar to your main application when you need PDF to text conversion but don't want to install weighty
applications inside your application environment.

Uses poppler to do the actual conversion.

## Using it

By default, pdf-bailiff exposes an HTTP server on port 3500.  Use the `PORT` environment variable to change that.

Send a POST request to the server with your PDF document as the body.  You'll either get back a 200 with the text,
or 500 if something went wrong.

There is a size limit of 5 MB.  If you exceed that you'll get a 413 error.

There is a processing time limit of 5 seconds to write the data and 5 seconds to read the text back from poppler.
If the request exceeds this time limit, a 500 will be returned.

```bash
  docker run --rm -p 3500:3500 ghcr.io/mogest/pdf-bailiff
  curl --data-binary @file.pdf localhost:3500
```

## License

MIT.  Copyright 2022 Mog Nesbitt.
