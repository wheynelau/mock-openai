FROM gcr.io/distroless/static-debian13:nonroot

ARG TARGETARCH
ARG BINARY_NAME

COPY artifacts/${TARGETARCH}/${BINARY_NAME} /usr/local/bin/${BINARY_NAME}

EXPOSE 8000

USER nonroot:nonroot

ENTRYPOINT ["${BINARY_NAME}"]
