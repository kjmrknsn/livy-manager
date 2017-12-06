pub const LOGIN: &'static str = r##"
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
        table {
            margin-top: 0.5rem;
        }
    </style>
  </head>
  <body>
    <nav class="navbar navbar-expand-md navbar-dark bg-dark fixed-top">
        <a class="navbar-brand" href="/">Livy Manager</a>
    </nav>

    <div class="container">
        <div class="row">
            <h4>Login</h4>
        </div>
        <div class="row">
            <form action="/login" method="post">
              <div class="form-group">
                <label for="uid">UID</label>
                <input type="text" class="form-control" id="uid" name="uid" aria-describedby="emailHelp" placeholder="UID">
              </div>
              <div class="form-group">
                <label for="password">Password</label>
                <input type="password" class="form-control" id="password" name="password" placeholder="Password">
              </div>
              <button type="submit" class="btn btn-primary">Submit</button>
            </form>
        </div>
    </div>

    <script src="https://code.jquery.com/jquery-3.2.1.min.js" integrity="sha256-hwg4gsxgFZhOsEEamdOYGBf13FyQuiTwlAQgxVSNgt4="
  crossorigin="anonymous"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/popper.js/1.12.3/umd/popper.min.js" integrity="sha384-vFJXuSJphROIrBnz7yo7oB41mKfc8JzQZiCq4NCceLEaO4IHwicKwpJf9c9IpFgh" crossorigin="anonymous"></script>
    <script src="https://maxcdn.bootstrapcdn.com/bootstrap/4.0.0-beta.2/js/bootstrap.min.js" integrity="sha384-alpBpkh1PFOepccYVYDB4do5UnbKysX5WZXm3XxPqe5iKTfUKjNkCk9SaVuEZflJ" crossorigin="anonymous"></script>
    <script type="text/javascript">
        $(function() {
            var queries = {};
            $.each(document.location.search.substr(1).split('&'),function(c,q){
              var i = q.split('=');
              if (i.length == 2) {
                queries[i[0].toString()] = i[1].toString();
              }
            });

            if (queries['result'] === 'failed') {
                alert('Authentication failed. Please make sure that the uid and password are correct.');
            }
        });
    </script>
  </body>
</html>
"##;
