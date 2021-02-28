# Alerter

There are two programs in this repository.

## `alert`

This is the command-line utility to initiate a message.

See `alert --help` on how to send messages.

This tool queues messages to the local `alerter`. Calls will never fail.

## `alerter`

This is the system daemon transmitting messages sent via `alert` to a backend.
Currently Slack and Matrix are supported. It reliably transfers messages and
retries failed transmission attempts.

The configuration file should be placed in `/var/lib/alerter/alerter.yml`. It
looks like this:

```yaml
socket_path: /tmp/.alerter.sock
spool_path: /var/lib/alerter/spool_queue
backend: ...
```

* `socket_path`: This is the location of the Unix Domain Socket between `alert`
  and the daemon. It should be a writable location for non-root users.

* `spool_path`: The file where faultily transmitted messages are persisted.

### Slack

```yaml
backend:
  slack:
    webhook: https://hooks.slack.com/services/...
```

See [Incoming WebHooks](https://slack.com/apps/A0F7XDUAZ-incoming-webhooks) on
how to generate a webhook URL.

### Matrix

```yaml
backend:
  matrix:
    user: user:homeserver.example
    password: changeme
    room: "!changeme:homeserver.example"
    message_template: ""
```

`user` and `password` belong to a genuine Matrix user. `room` is the default
room ID to send to if `alert` doesn't set one.

`message_template` is the HTML template used to render the message. It uses
the [tera](https://tera.netlify.app/) template engine. A sane default is
provided but you are free to change it.