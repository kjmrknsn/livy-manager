pub const INDEX: &'static str = r#"
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
            <h4>Active Sessions</h4>
            <table class="table table-hover table-sm">
              <thead class="thead-light">
                <tr>
                  <th scope="col">ID</th>
                  <th scope="col">App ID</th>
                  <th scope="col">User</th>
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

    <script src="https://code.jquery.com/jquery-3.2.1.slim.min.js" integrity="sha384-KJ3o2DKtIkvYIK3UENzmM7KCkRr/rE9/Qpg6aAZGJwFDMVNA/GpGFF93hXpG5KkN" crossorigin="anonymous"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/popper.js/1.12.3/umd/popper.min.js" integrity="sha384-vFJXuSJphROIrBnz7yo7oB41mKfc8JzQZiCq4NCceLEaO4IHwicKwpJf9c9IpFgh" crossorigin="anonymous"></script>
    <script src="https://maxcdn.bootstrapcdn.com/bootstrap/4.0.0-beta.2/js/bootstrap.min.js" integrity="sha384-alpBpkh1PFOepccYVYDB4do5UnbKysX5WZXm3XxPqe5iKTfUKjNkCk9SaVuEZflJ" crossorigin="anonymous"></script>
  </body>
</html>
"#;
