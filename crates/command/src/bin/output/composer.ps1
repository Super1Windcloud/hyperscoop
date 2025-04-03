# A:\Scoop\apps\composer\current\composer.ps1
$path = "A:\Scoop\apps\composer\current\composer.ps1"
if ($MyInvocation.ExpectingInput) { $input | & A:\Scoop\apps\composer\current\composer.ps1  @args } else { & A:\Scoop\apps\composer\current\composer.ps1 @args }
exit $LASTEXITCODE
