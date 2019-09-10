package main

import (
	"fmt"
	"io/ioutil"
	"os"
	"strings"
)


func replace(file string) {
	buf, err := ioutil.ReadFile(file)
	if err != nil {
		fmt.Printf("read file:%s error: %v\n", file, err)
		return
	}
	content := string(buf)

	pat:=`include!(concat!(env!("OUT_DIR"), "`
	if strings.Contains(content, pat) == false {
		return
	}
	for {
		pos := strings.Index(content, pat)
		if pos == -1 {
			break
		}
		rep := content[pos+len(pat):]
		sp := strings.Split(rep, "\"")
		replaceFile := "./target/debug/build/cranelift-codegen-ba4dc72176f6ae31/out/" + sp[0]

		replace, err := ioutil.ReadFile(replaceFile)
		if err != nil {
			fmt.Printf("read file:%s error: %v\n", file, err)
			return
		}

		content = fmt.Sprintf("%s \n%s\n //%s", content[:pos], replace, content[pos+2:])
	}

	filename := file + "~"
	err  = ioutil.WriteFile(filename, []byte(content), 0644)
	if err != nil {
		fmt.Printf("write file:%s error: %v\n", file, err)
		return
	}
	os.Rename(filename, file)

}

func main() {
	if len(os.Args) < 2 {
		fmt.Printf("usage: %s originfile, got: %v", os.Args[0], os.Args)
		return
	}

	for _, file := range os.Args[1:] {
		replace(file)
	}
}
