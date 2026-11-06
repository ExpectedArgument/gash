# wait reports 128+signal for a job stopped with kill, as bash does.
sleep 30 & p=$!; kill $p; wait $p; echo "TERM: $?"
sleep 30 & p=$!; kill -9 $p; wait $p 2>/dev/null; echo "KILL: $?"
sleep 30 & p=$!; kill -s HUP $p; wait $p; echo "HUP: $?"
sleep 30 & kill %1; wait %1; echo "job spec: $?"
ping -n 30 127.0.0.1 > /dev/null & p=$!; kill $p; wait $p; echo "native program: $?"
