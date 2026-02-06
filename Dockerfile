FROM gcr.io/distroless/static-debian13:nonroot

ARG REPO=wheynelau/mock-openai
ARG TARGETARCH

COPY artifacts/${TARGETARCH}/mock-openai /usr/local/bin/mock-openai

EXPOSE 8000

USER nonroot:nonroot

ENTRYPOINT ["/mock-openai"]
