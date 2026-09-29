// scandir_count.go - Example demonstrating the Count function for fast statistics.
package main

import (
	"fmt"
	"os"

	cscandir "github.com/brmmm3/scandir-rs/goscandir"
)

func main() {
	root := "."
	if len(os.Args) > 1 {
		root = os.Args[1]
	}

	options := &cscandir.Options{
		SkipHidden: true,
	}

	stats, err := cscandir.Count(root, options)
	if err != nil {
		fmt.Fprintf(os.Stderr, "error: %v\n", err)
		os.Exit(1)
	}

	fmt.Println("Statistics:")
	fmt.Printf("  Directories: %d\n", stats.Dirs)
	fmt.Printf("  Files: %d\n", stats.Files)
	fmt.Printf("  Symlinks: %d\n", stats.Slinks)
	fmt.Printf("  Hardlinks: %d\n", stats.Hlinks)
	fmt.Printf("  Devices: %d\n", stats.Devices)
	fmt.Printf("  Pipes: %d\n", stats.Pipes)
	fmt.Printf("  Total size: %d bytes\n", stats.Size)
	fmt.Printf("  Disk usage: %d bytes\n", stats.Usage)
	fmt.Printf("  Duration: %.4fs\n", stats.Duration)
}