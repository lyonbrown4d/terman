package store

// RemoveByPID deletes records belonging to a session server, including records
// whose names changed while that server was running.
func RemoveByPID(pid int) error {
	lock, err := lockStore()
	if err != nil {
		return err
	}
	defer unlockStore(lock)
	records, err := load()
	if err != nil {
		return err
	}
	for _, record := range records {
		if pid > 0 && record.PID == pid {
			if err := remove(record.Name); err != nil {
				return err
			}
		}
	}
	return nil
}
