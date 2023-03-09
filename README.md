<h1 align="center">Brickhill friend bot</h1>
<h2>So what does it do?</h2>
<ul>
  <li>Goes through user ids incrementally</li>
  <li>If it reaches an account that hasn't been created yet it will wait until it is created.</li>
  <li>Stores users it has friended in users.json</li>
</ul>

<h2>How to setup?</h2>
<ol>
  <li><a href="https://chromedriver.chromium.org/downloads">Download chromedriver</a>. Make sure you choose the right version, you can find your chrome version from chrome://version/ </li>
  <li>Open and make sure chromedriver is running on port 9515 (Default port)</li>
  <li>Run this program</li>
</ol>

<h2>Usage</h2>

```
USAGE:
    brickhill_friend_bot [OPTIONS] --username <USERNAME> --password <PASSWORD>

OPTIONS:
        --chromedriver-port <CHROMEDRIVER_PORT>
            Chromedriver port [default: 9515]

        --discord-webhook-url <DISCORD_WEBHOOK_URL>
            Discord webhook to send updates to

        --error-wait-time <ERROR_WAIT_TIME>
            Wait time after error [default: 5000]

    -f, --file <FILE>
            [default: users.json]

    -h, --help
            Print help information

    -p, --password <PASSWORD>
            Login password

        --start-id <START_ID>
            User ID to start from [default: 364614]

    -u, --username <USERNAME>
            Login username

    -V, --version
            Print version information

        --wait-max <WAIT_MAX>
            Max amount of time to wait between tries [default: 3000]

        --wait-min <WAIT_MIN>
            Minimum amount of time to wait between tries [default: 1000]
```
