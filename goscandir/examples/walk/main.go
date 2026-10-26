// Example demonstrating the Walk function.
package main

import (
	"fmt"
	"os"

	cscandir "github.com/brmmm3/scandir-rs/goscandir"
)

func main() {
	root, ok := parseRoot()
	if !ok {
		fmt.Fprintf(os.Stderr, "Usage: %s [ROOT]\n", os.Args[0])
		os.Exit(2)
	}

	// Walk itself follows the supplied options; set SkipHidden to true here to
	// match the Rust Walk example's default.
	options := &cscandir.Options{
		SkipHidden: true,
		Sorted:     true,
	}

	result, err := cscandir.Walk(root, options)
	if err != nil {
		fmt.Fprintf(os.Stderr, "error: %v\n", err)
		os.Exit(1)
	}

	fmt.Printf("Merged TOC: dirs=%d files=%d symlinks=%d other=%d\n",
		len(result.Toc.Dirs), len(result.Toc.Files),
		len(result.Toc.Symlinks), len(result.Toc.Other))
	fmt.Printf("Directories visited: %d\n", len(result.Roots))
	fmt.Printf("Collect duration: %.4fs\n\n", result.Statistics.Duration)

	for _, entry := range result.Roots {
		name := entry.Dir
		if name == "" {
			name = "."
		}
		fmt.Printf("%s: dirs=%d files=%d symlinks=%d other=%d\n",
			name,
			len(entry.Toc.Dirs), len(entry.Toc.Files),
			len(entry.Toc.Symlinks), len(entry.Toc.Other))
	}

	if len(result.Toc.Errors) > 0 {
		fmt.Println("\nScan errors:")
		for _, err := range result.Toc.Errors {
			fmt.Printf("  %s\n", err)
		}
		os.Exit(1)
	}
}

func parseRoot() (string, bool) {
	if len(os.Args) > 2 {
		return "", false
	}
	if len(os.Args) == 2 {
		return os.Args[1], true
	}
	return ".", true
}
