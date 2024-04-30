cargo build
mellon="./target/debug/mellon"

token=$($mellon token add testing_token)
echo "Got temporary token: $token"

nohup $mellon serve &
SERVER_PID=$!

# Wait for the server to start
sleep 5

verbs=("GET" "POST" "PUT" "DELETE" "PATCH" "HEAD" "OPTIONS" "TRACE" "CONNECT")

for verb in "${verbs[@]}"; do
	response=$(curl -o /dev/null -s -w "%{http_code}" -X "$verb" localhost:8090/auth -H "Authorization: Bearer $token")
	if [[ "$response" == "200" ]]; then
		echo "Success with $verb: HTTP response is 200"
	else
		echo "Error with $verb: HTTP response is not 200, it is $response"
	fi
done

# remove the token
$mellon token rescind testing_token

# stop the server
kill $SERVER_PID
