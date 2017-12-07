# Livy Manager
[![Build Status](https://travis-ci.org/kjmrknsn/livy-manager.svg?branch=master)](https://travis-ci.org/kjmrknsn/livy-manager)

## Abstract
Livy Manager is a Web UI for managing [Apache Livy](https://livy.incubator.apache.org/) sessions.
![](https://raw.githubusercontent.com/kjmrknsn/livy-manager/master/img/livy-manager.png)

## Issues Livy Manager Tackles with
* It is difficult for non-developer Livy users to monitor or kill their Livy sessions and Spark applications.
    * They have to use a HTTP client tool like `curl` to call the Livy REST APIs.
    * Additionally, they have to manipulate machines on which a Kerberos client is installed if the Livy service is Kerberized.
    * In some services which use Livy, there's no way to kill a Livy sessions while a Spark application is running, so users cannot stop their Spark applications when they submitted a heavy and long running application accidentally.

## Solutions Livy Manager provides
* Non-developer Livy users can see and kill their Livy sessions.
* Optional LDAP authentication and authorization feature is included.
    * Admin users can see and kill all of the Livy sessions.
    * Non-admin users can see and kill only their Livy sessions.
    * This feature works well with Zeppelin with LDAP authentication and the Livy interpreter.

## Setup
1. Download an executable binary file from the [Releases](https://github.com/kjmrknsn/livy-manager/releases) page and deploy it to your server.
2. Deploy a Livy Manager configuration file to your server. Please see [conf/livy-manager.toml.template](https://github.com/kjmrknsn/livy-manager/blob/master/conf/livy-manager.toml.template) for its template.
3. Run Livy Manager by executing the following command:
```bash
$ /path/to/livy-mangager -c /path/to/livy-manager-configuration-file
```
