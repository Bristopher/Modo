{{/*
SPDX-License-Identifier: AGPL-3.0-or-later
*/}}

{{- define "svc-common.deployment" -}}
apiVersion: apps/v1
kind: Deployment
metadata:
  name: {{ .Values.svc.name }}
  namespace: {{ .Values.global.namespace }}
  labels:
    {{- include "svc-common.labels" . | nindent 4 }}
spec:
  replicas: {{ .Values.svc.router.replicas }}
  minReadySeconds: {{ default 10 .Values.svc.router.minReadySeconds }}
  selector:
    matchLabels:
      app: {{ .Values.svc.name }}
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: {{ default 1 .Values.svc.router.maxSurge }}
      maxUnavailable: {{ default 0 .Values.svc.router.maxUnavailable }}
  template:
    metadata:
      labels:
        app: {{ .Values.svc.name }}
        {{- include "svc-common.labels" . | nindent 8 }}
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "{{ .Values.svc.router.port }}"
        prometheus.io/path: "/_metrics"
    spec:
      terminationGracePeriodSeconds: {{ default 60 .Values.svc.router.terminationGracePeriodSeconds }}
      securityContext:
        runAsNonRoot: true
        seccompProfile:
          type: RuntimeDefault
      containers:
        - name: router
          image: {{ include "svc-common.image" . }}
          securityContext:
            allowPrivilegeEscalation: false
            readOnlyRootFilesystem: false
          ports:
            - name: http
              containerPort: {{ .Values.svc.router.port }}
          env:
            - name: FLUXER_SVC_MODE
              value: "router"
            - name: FLUXER_SVC_NAME
              value: {{ .Values.svc.name }}
            - name: FLUXER_SVC_SHARD_COUNT
              value: {{ .Values.svc.shard.replicas | quote }}
            - name: FLUXER_SVC_PORT
              value: {{ .Values.svc.router.port | quote }}
            - name: FLUXER_SVC_NATS_URL
              value: {{ .Values.svc.nats.url }}
            - name: FLUXER_SVC_CACHE_MAX_ENTRIES
              value: {{ .Values.svc.cache.maxEntries | quote }}
            - name: FLUXER_SVC_CACHE_TTL_MS
              value: {{ .Values.svc.cache.ttlMs | quote }}
            {{- if .Values.svc.build }}
            - name: BUILD_VERSION
              value: {{ .Values.svc.build.version | default .Values.svc.tag | quote }}
            - name: RELEASE_CHANNEL
              value: {{ .Values.svc.build.channel | default "stable" | quote }}
            {{- end }}
            {{- range .Values.svc.extraEnv }}
            - name: {{ .name }}
              {{- if .valueFrom }}
              valueFrom:
                {{- toYaml .valueFrom | nindent 16 }}
              {{- else }}
              value: {{ .value | quote }}
              {{- end }}
            {{- end }}
          readinessProbe:
            httpGet:
              path: /_health
              port: http
            initialDelaySeconds: 1
            periodSeconds: 5
          livenessProbe:
            httpGet:
              path: /_healthz
              port: http
            initialDelaySeconds: 2
            periodSeconds: 15
          resources:
            {{- toYaml .Values.svc.router.resources | nindent 12 }}
      {{- if .Values.global.imagePullSecret }}
      imagePullSecrets:
        - name: {{ .Values.global.imagePullSecret }}
      {{- end }}
{{- end -}}
