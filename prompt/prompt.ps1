# CMD Version output to look just like CMD
[array] $ver = cmd /c ver
"$($ver[1])`n(c) 2020 Microsoft Corporation. All rights reserved.`n"

# set up required vars
[String]$finexe = "$PSSCriptRoot\environment_saver.exe"
[int]$pipecode = Get-Random
[bool]$firstrun = $true
# Set up env vars
$env:TERM = "xterm-256color"
$env:cerrcode = "0"

# start the server
cmd /c start /min /high /b "" $finexe --server -i $pipecode --hide-output


function ShellLoop {
    try {
        while($true) {
            if ($env:cerrcode -eq $null) {
                # Execution aborted early due to an error ("was unexpected at this time."), so fix code
                # error used for testing: if [not] ERRORLEVEL
                $env:cerrcode = "1"
            }

            powerline-go -shell bare -colorize-hostname -error $env:cerrcode -newline

            # job takes long to startup. this is a performance improvement

            # reset for next iteration
            [String]$cmd = ""
            $env:cerrcode = ""

            # cache old var data for comparison; done here for performance ; cmd is actually faster than ps native env display!
            cmd /c "set | $finexe --client -i $pipecode -o"

            [String]$cmd = Read-Host

            if ($cmd -eq "exit") {
                exit
            }

            if ($cmd -eq "") {
                $env:cerrcode = "0"
                continue
            } elseif ($cmd.Substring(0,2) -eq "cd") {
                # no real other way to get the success state
                $cmd += ';$succ=$?'
                Invoke-Expression $cmd 2>&1 | Out-Null

                $env:cerrcode = "0"

                if (-Not $succ) {
                    # pretend we were in cmd lol
                    "The system cannot find the path specified."
                    $env:cerrcode = "1"
                }
            } else {
                # execute command in command prompt!
                cmd /V:ON /c "$cmd & set | $finexe --client -i $pipecode -e !errorlevel! -s"
                $res = Invoke-Expression "$finexe --client -i $pipecode -r"
                foreach ($line in $res.Split("`n")) {
                    if ($line.length -gt 0) {
                        Invoke-Expression $line
                    }
                }
            }
        }
    } finally {
        # call itself again
        # this line is only caught when ^C was pressed
        Write-Host "^C"
        $env:cerrcode = "0"
        ShellLoop
    }
}

# begin execution
ShellLoop
