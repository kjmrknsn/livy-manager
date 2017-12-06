pub const INDEX: &'static str = r##"
<!doctype html>
<html lang="en">
  <head>
    <title>Livy Manager</title>

    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1, shrink-to-fit=no">
    <meta content="IE=edge" http-equiv="X-UA-Compatible">

    <link rel="stylesheet" href="https://maxcdn.bootstrapcdn.com/bootstrap/4.0.0-beta.2/css/bootstrap.min.css" integrity="sha384-PsH8R72JQ3SOdhVi3uxftmaW6Vc51MKb0q5P2rRUpPvrszuE4W1povHYgTpBfshb" crossorigin="anonymous">
    <style type="text/css">
        body {
            padding-top: 5rem;
        }
        .navbar-brand {
            font-size: 1.5rem;
        }
        .navbar-text {
            padding-left: 0.5rem;
            padding-right: 0.5rem;
        }
        table {
            margin-top: 0.5rem;
        }
    </style>
  </head>
  <body>
    <nav class="navbar navbar-expand-md navbar-dark bg-dark fixed-top">
        <a class="navbar-brand" href="/">Livy Manager</a>
        <div class="collapse navbar-collapse" id="navbar">
            <ul class="navbar-nav mr-auto"></ul>
            <div class="navbar-nav navbar-right">
                <div id="uid" class="navbar-text"></div>
                <a class="nav-link" href="/logout">Log Out</a>
            </div>
        </div>
    </nav>

    <div class="container">
        <div class="row">
            <h4>Active Sessions</h4>
            <table class="table table-hover table-sm">
              <thead class="thead-light">
                <tr>
                  <th scope="col">ID</th>
                  <th scope="col">App ID</th>
                  <th scope="col">Proxy User</th>
                  <th scope="col">Kind</th>
                  <th scope="col">State</th>
                  <th scope="col">Operation</th>
                </tr>
              </thead>
              <tbody id="sessions">
              </tbody>
            </table>
        </div>
    </div>

    <script src="https://code.jquery.com/jquery-3.2.1.min.js" integrity="sha256-hwg4gsxgFZhOsEEamdOYGBf13FyQuiTwlAQgxVSNgt4="
  crossorigin="anonymous"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/popper.js/1.12.3/umd/popper.min.js" integrity="sha384-vFJXuSJphROIrBnz7yo7oB41mKfc8JzQZiCq4NCceLEaO4IHwicKwpJf9c9IpFgh" crossorigin="anonymous"></script>
    <script src="https://maxcdn.bootstrapcdn.com/bootstrap/4.0.0-beta.2/js/bootstrap.min.js" integrity="sha384-alpBpkh1PFOepccYVYDB4do5UnbKysX5WZXm3XxPqe5iKTfUKjNkCk9SaVuEZflJ" crossorigin="anonymous"></script>
    <script type="text/javascript">
        function isEmpty(o) {
            return o === null || o === undefined || o === '';
        }

        function fmtStr(o) {
            var s = $.trim(o);
            if (s === '') {
                return '-';
            }
            return s;
        }

        function appIdLink(appId, appInfo) {
            appId = $.trim(appId);

            if (isEmpty(appId)) {
                return '-';
            }

            if (isEmpty(appInfo)) {
                return appId;
            }

            var sparkUiUrl = $.trim(appInfo.sparkUiUrl);

            if (isEmpty(sparkUiUrl)) {
                return appId;
            }

            return '<a href="' + sparkUiUrl + '" target="_blank"> ' + appId + '</a>';
        }

        function killLink(id) {
            id = $.trim(id);

            if (isEmpty(id)) {
                return '';
            }

            return '<a href="#" onclick="killSession(\'' + id + '\');">kill</a>';
        }

        function killSession(id) {
            if (!confirm('Are you sure to kill the session ' + id + '?')) {
                return;
            }

            $.ajax({
                url: '/api/sessions/' + id,
                method: 'DELETE',
                contentType: 'application/json',
            }).done(function() {
                alert('Session ' + id + ' was killed successfully.');
                location.href = '/';
            }).fail(function(d) {
                alert('Failed to kill the session.');
            });
        }

        $(function() {
            $.getJSON(
                '/api/sessions'
            ).done(function(sessions) {
                $.each(sessions, function(_, session) {
                    $('#sessions').append(
                        '<tr>' +
                            '<td>' + fmtStr(session.id)                           + '</td>' +
                            '<td>' + appIdLink(session.appId, session.appInfo)    + '</td>' +
                            '<td>' + fmtStr(session.proxyUser)                    + '</td>' +
                            '<td>' + fmtStr(session.kind)                         + '</td>' +
                            '<td>' + fmtStr(session.state)                        + '</td>' +
                            '<td>' + killLink(session.id)                         + '</td>' +
                        '</tr>');
                });
            });

            $.getJSON(
                '/api/uid'
            ).done(function(uid) {
                $('#uid').text(uid);
            });
        });
    </script>
  </body>
</html>
"##;
