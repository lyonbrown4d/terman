package app

func (s *state) processHeaderLine(width int) string {
	values := []string{
		s.sortLabel(SortPID, "PID"),
		s.sortLabel(SortUser, "USER"),
		s.sortLabel(SortCPU, "CPU%"),
		s.sortLabel(SortMemory, "MEM"),
		s.sortLabel(SortIO, "IO/s"),
		"S",
		s.sortLabel(SortName, "COMMAND"),
	}
	line := formatColumns(
		values,
		[]int{9, 12, 8, 10, 10, 2, max(1, width-57)},
	)
	return fit(line, width)
}

func (s *state) sortLabel(key SortKey, label string) string {
	if s.sort != key {
		return label
	}
	descending := key == SortCPU || key == SortMemory || key == SortIO
	if s.reverse {
		descending = !descending
	}
	if descending {
		return label + "v"
	}
	return label + "^"
}
