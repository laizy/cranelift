go build -o replace.exe ./replace_include.go

fd \.rs$ | xargs ./replace.exe
