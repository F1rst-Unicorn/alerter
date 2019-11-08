# Alerter

There are two programs in this repository.

## `alert`

This is the command-line utility to initiate a message.

See `alert --help` on how to send messages.

This tool queues messages to the local `alerter`. Calls will never fail.

## `alerter`

This is the system daemon transmitting messages sent via `alert` to a backend.
Currently only Slack is supported. It reliably transfers messages and retries
failed transmission attempts.

The configuration file should be placed in `/var/lib/alerter/alerter.yml`. It
looks like this:

```yaml
socket_path: /tmp/.alerter.sock
spool_path: /var/lib/alerter/spool_queue
webhook: https://hooks.slack.com/services/xxx/xxx/xxx
```

* `socket_path`: This is the location of the Unix Domain Socket between `alert`
  and the daemon. It should be a writable location for non-root users.

* `spool_path`: The file where faultily transmitted messages are persisted.

* `webhook`: The slack webhook to send to. See [Incoming
  WebHooks](https://slack.com/apps/A0F7XDUAZ-incoming-webhooks) for details.
