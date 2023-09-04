package roundtrip

import (
	"errors"
	"fmt"
	"io"
	"net/http"
	"strconv"
	"strings"

	go_wasi_http "pack/name/component_name"
)

type WasiHttpTransport struct {
}

func (t WasiHttpTransport) RoundTrip(request *http.Request) (*http.Response, error) {

	var headerKeyValues []go_wasi_http.WasiHttpTypesTuple2StringStringT
	for key, values := range request.Header {
		for _, value := range values {
			headerKeyValues = append(headerKeyValues, go_wasi_http.WasiHttpTypesTuple2StringStringT{
				F0: key,
				F1: value,
			})
		}
	}
	headers := go_wasi_http.WasiHttpTypesNewFields(headerKeyValues)
	defer go_wasi_http.WasiHttpTypesDropFields(headers)

	var method go_wasi_http.WasiHttpTypesMethod
	switch strings.ToUpper(request.Method) {
	case "":
		method = go_wasi_http.WasiHttpTypesMethodGet()
	case "GET":
		method = go_wasi_http.WasiHttpTypesMethodGet()
	case "HEAD":
		method = go_wasi_http.WasiHttpTypesMethodHead()
	case "POST":
		method = go_wasi_http.WasiHttpTypesMethodPost()
	case "PUT":
		method = go_wasi_http.WasiHttpTypesMethodPut()
	case "DELETE":
		method = go_wasi_http.WasiHttpTypesMethodDelete()
	case "CONNECT":
		method = go_wasi_http.WasiHttpTypesMethodConnect()
	case "OPTIONS":
		method = go_wasi_http.WasiHttpTypesMethodOptions()
	case "TRACE":
		method = go_wasi_http.WasiHttpTypesMethodTrace()
	case "PATCH":
		method = go_wasi_http.WasiHttpTypesMethodPatch()
	default:
		method = go_wasi_http.WasiHttpTypesMethodOther(request.Method)
	}

	path := request.URL.Path
	query := request.URL.RawQuery
	pathAndQuery := path
	if query != "" {
		pathAndQuery += "?" + query
	}

	var scheme go_wasi_http.WasiHttpTypesScheme
	switch strings.ToLower(request.URL.Scheme) {
	case "http":
		scheme = go_wasi_http.WasiHttpTypesSchemeHttp()
	case "https":
		scheme = go_wasi_http.WasiHttpTypesSchemeHttps()
	default:
		scheme = go_wasi_http.WasiHttpTypesSchemeOther(request.URL.Scheme)
	}

	userPassword := request.URL.User.String()
	var authority string
	if userPassword == "" {
		authority = request.URL.Host
	} else {
		authority = userPassword + "@" + request.URL.Host
	}

	requestHandle := go_wasi_http.WasiHttpTypesNewOutgoingRequest(
		method,
		go_wasi_http.Some[string](pathAndQuery),
		go_wasi_http.Some[go_wasi_http.WasiHttpTypesScheme](scheme),
		go_wasi_http.Some[string](authority),
		headers,
	)
	defer go_wasi_http.WasiHttpTypesDropOutgoingRequest(requestHandle)

	if request.Body != nil {
		reader := request.Body
		defer reader.Close()

		requestBodyResult := go_wasi_http.WasiHttpTypesOutgoingRequestWrite(requestHandle)
		if requestBodyResult.IsErr() {
			return nil, errors.New("Failed to start writing request body")
		}
		requestBody := requestBodyResult.Unwrap()

		buffer := make([]byte, 1024)
		for {
			n, err := reader.Read(buffer)

			result := go_wasi_http.WasiIoStreamsWrite(requestBody, buffer[:n])
			if result.IsErr() {
				go_wasi_http.WasiIoStreamsDropOutputStream(requestBody)
				return nil, errors.New("Failed to write request body chunk")
			}

			if err == io.EOF {
				break
			}
		}

		go_wasi_http.WasiHttpTypesFinishOutgoingStream(requestBody, go_wasi_http.None[uint32]())
		go_wasi_http.WasiIoStreamsDropOutputStream(requestBody)
	}

	// TODO: timeouts
	connectTimeoutMs := go_wasi_http.None[uint32]()
	firstByteTimeoutMs := go_wasi_http.None[uint32]()
	betweenBytesTimeoutMs := go_wasi_http.None[uint32]()
	options := go_wasi_http.WasiHttpTypesRequestOptions{
		ConnectTimeoutMs:      connectTimeoutMs,
		FirstByteTimeoutMs:    firstByteTimeoutMs,
		BetweenBytesTimeoutMs: betweenBytesTimeoutMs,
	}

	future := go_wasi_http.WasiHttpOutgoingHandlerHandle(requestHandle, go_wasi_http.Some(options))
	defer go_wasi_http.WasiHttpTypesDropFutureIncomingResponse(future)

	incomingResponse, err := GetIncomingResponse(future)
	if err != nil {
		return nil, err
	}

	status := go_wasi_http.WasiHttpTypesIncomingResponseStatus(incomingResponse)
	responseHeaders := go_wasi_http.WasiHttpTypesIncomingResponseHeaders(incomingResponse)
	defer go_wasi_http.WasiHttpTypesDropFields(responseHeaders)

	responseHeaderEntries := go_wasi_http.WasiHttpTypesFieldsEntries(responseHeaders)
	header := http.Header{}

	for _, tuple := range responseHeaderEntries {
		ck := http.CanonicalHeaderKey(tuple.F0)
		header[ck] = append(header[ck], string(tuple.F1))
	}

	var contentLength int64
	clHeader := header.Get("Content-Length")
	switch {
	case clHeader != "":
		cl, err := strconv.ParseInt(clHeader, 10, 64)
		if err != nil {
			return nil, fmt.Errorf("net/http: ill-formed Content-Length header: %v", err)
		}
		if cl < 0 {
			// Content-Length values less than 0 are invalid.
			// See: https://datatracker.ietf.org/doc/html/rfc2616/#section-14.13
			return nil, fmt.Errorf("net/http: invalid Content-Length header: %q", clHeader)
		}
		contentLength = cl
	default:
		// If the response length is not declared, set it to -1.
		contentLength = -1
	}

	responseBodyStreamResult := go_wasi_http.WasiHttpTypesIncomingResponseConsume(incomingResponse)
	if responseBodyStreamResult.IsErr() {
		return nil, errors.New("Failed to consume response body")
	}
	responseBodyStream := responseBodyStreamResult.Unwrap()

	responseReader := WasiStreamReader{
		Handle: responseBodyStream,
	}

	response := http.Response{
		Status:        fmt.Sprintf("%d %s", status, http.StatusText(int(status))),
		StatusCode:    int(status),
		Header:        header,
		ContentLength: contentLength,
		Body:          responseReader,
		Request:       request,
	}

	return &response, nil
}

func GetIncomingResponse(future uint32) (uint32, error) {
	result := go_wasi_http.WasiHttpTypesFutureIncomingResponseGet(future)
	if result.IsSome() {
		result2 := result.Unwrap()
		if result2.IsErr() {
			return 0, errors.New("Failed to send request")
		}
		return result2.Unwrap(), nil
	} else {
		pollable := go_wasi_http.WasiHttpTypesListenToFutureIncomingResponse(future)
		var pollables []uint32
		pollables = append(pollables, pollable)
		go_wasi_http.WasiPollPollPollOneoff(pollables)
		go_wasi_http.WasiPollPollDropPollable(pollable)
		return GetIncomingResponse(future)
	}
}

type WasiStreamReader struct {
	Handle uint32
}

func (reader WasiStreamReader) Read(p []byte) (int, error) {
	c := cap(p)
	result := go_wasi_http.WasiIoStreamsRead(reader.Handle, uint64(c))
	if result.IsErr() {
		return 0, errors.New("Failed to read response stream")
	}

	tuple := result.Unwrap()
	var err error
	if tuple.F1 == go_wasi_http.WasiIoStreamsStreamStatusEnded() {
		err = io.EOF
	} else {
		err = nil
	}

	chunk := tuple.F0
	copy(p, chunk)
	return len(chunk), err
}

func (reader WasiStreamReader) Close() error {
	go_wasi_http.WasiIoStreamsDropInputStream(reader.Handle)
	return nil
}
