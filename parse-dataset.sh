echo "Amount of characters of each numerical entry of column 5:"
awk -vFPAT='([^,]*)|("[^"]+")' '{print $5}' dataset/DT-mobile-data.csv | sort | uniq | grep -Eo "^.+\..*$" | awk '{print length}' | sort | uniq

echo "Short, unique final 5 characters:"
awk -vFPAT='([^,]*)|("[^"]+")' '{print $5}' dataset/DT-mobile-data.csv | sort | uniq | grep -Eo "^.\..*$" | cut -c 7- | sort | uniq

echo "Long, unique final 4 characters:"
awk -vFPAT='([^,]*)|("[^"]+")' '{print $5}' dataset/DT-mobile-data.csv | sort | uniq | grep -Eo "^..\..*$" | cut -c 8- | sort | uniq

echo "Amount of characters of each numerical entry of column 6:"
awk -vFPAT='([^,]*)|("[^"]+")' '{print $6}' dataset/DT-mobile-data.csv | sort | uniq | grep -Eo '^.+\..*$' | awk '{print length}' | sort | uniq

echo "Short, unique final 5 characters:"
awk -vFPAT='([^,]*)|("[^"]+")' '{print $6}' dataset/DT-mobile-data.csv | sort | uniq | grep -Eo "^.\..*$"

echo "Long, unique final 4 characters:"
awk -vFPAT='([^,]*)|("[^"]+")' '{print $6}' dataset/DT-mobile-data.csv | sort | uniq | grep -Eo "^..\..*$" | cut -c 8- | sort | uniq

echo "Unique positions:"
awk -vFPAT='([^,]*)|("[^"]+")' '{print $6}' dataset/DT-mobile-data.csv | sort | uniq | grep -Eo "^..\..*$" | sort | uniq | wc -l
