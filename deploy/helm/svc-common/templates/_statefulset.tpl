{{/*
SPDX-License-Identifier: AGPL-3.0-or-later
*/}}

{{- define "svc-common.statefulset" -}}
{{- $persistence := .Values.svc.shard.persistence | default dict -}}
{{- $ephemeral := .Values.svc.shard.ephemeral | default dict -}}
{{- $dataMountEnabled := or $persistence.enabled $ephemeral.enabled -}}
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: {{ .Values.svc.name }}-shard
  namespace: {{ .Values.global.namespace }}
  labels:
    {{- include "svc-common.labels" . | nindent 4 }}
spec:
  serviceName: {{ .Values.svc.name }}-shard-headless
  replicas: {{ .Values.svc.shard.replicas }}
  minReadySeconds: {{ default 10 .Values.svc.shard.minReadySeconds }}
  podManagementPolicy: Parallel
  updateStrategy:
    type: RollingUpdate
  selector:
    matchLabels:
      app: {{ .Values.svc.name }}-shard
  template:
    metadata:
      labels:
        app: {{ .Values.svc.name }}-shard
        {{- include "svc-common.labels" . | nindent 8 }}
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "{{ .Values.svc.shard.port }}"
        prometheus.io/path: "/_metrics"
    spec:
      terminationGracePeriodSeconds: {{ default 60 .Values.svc.shard.terminationGracePeriodSeconds }}
      securityContext:
        runAsNonRoot: true
        fsGroup: 65532
        seccompProfile:
          type: RuntimeDefault
      containers:
        - name: shard
          image: {{ include "svc-common.image" . }}
          securityContext:
            allowPrivilegeEscalation: false
            readOnlyRootFilesystem: false
          ports:
            - name: http
              containerPort: {{ .Values.svc.shard.port }}
          env:
            - name: FLUXER_SVC_MODE
              value: "shard"
            - name: FLUXER_SVC_NAME
              value: {{ .Values.svc.name }}
            - name: FLUXER_SVC_SHARD_COUNT
              value: {{ .Values.svc.shard.replicas | quote }}
            - name: FLUXER_SVC_PORT
              value: {{ .Values.svc.shard.port | quote }}
            - name: FLUXER_SVC_NATS_URL
              value: {{ .Values.svc.nats.url }}
            - name: POD_NAME
              valueFrom:
                fieldRef:
                  fieldPath: metadata.name
            {{- if .Values.svc.cassandra }}
            - name: FLUXER_CASSANDRA_HOSTS
              value: {{ join "," .Values.svc.cassandra.hosts }}
            - name: FLUXER_CASSANDRA_KEYSPACE
              value: {{ .Values.svc.cassandra.keyspace }}
            {{- if .Values.svc.cassandra.credentialsSecret }}
            - name: FLUXER_CASSANDRA_USERNAME
              valueFrom:
                secretKeyRef:
                  name: {{ .Values.svc.cassandra.credentialsSecret }}
                  key: username
            - name: FLUXER_CASSANDRA_PASSWORD
              valueFrom:
                secretKeyRef:
                  name: {{ .Values.svc.cassandra.credentialsSecret }}
                  key: password
            {{- end }}
            {{- end }}
            - name: FLUXER_SVC_CACHE_MAX_ENTRIES
              value: {{ .Values.svc.cache.maxEntries | quote }}
            - name: FLUXER_SVC_CACHE_TTL_MS
              value: {{ .Values.svc.cache.ttlMs | quote }}
            - name: FLUXER_SVC_MAX_CONCURRENT_REQUESTS
              value: {{ default 64 .Values.svc.maxConcurrentRequests | quote }}
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
            initialDelaySeconds: 2
            periodSeconds: 5
            failureThreshold: 2
          livenessProbe:
            httpGet:
              path: /_healthz
              port: http
            initialDelaySeconds: 5
            periodSeconds: 15
            failureThreshold: 3
          startupProbe:
            httpGet:
              path: /_healthz
              port: http
            initialDelaySeconds: 1
            periodSeconds: 5
            failureThreshold: 60
          resources:
            {{- toYaml .Values.svc.shard.resources | nindent 12 }}
          {{- if $dataMountEnabled }}
          volumeMounts:
            - name: data
              mountPath: {{ default (default "/var/lib/fluxer-svc" $persistence.mountPath) $ephemeral.mountPath }}
          {{- end }}
      {{- with .Values.svc.nodeSelector }}
      nodeSelector: {{- toYaml . | nindent 8 }}
      {{- end }}
      {{- with .Values.svc.tolerations }}
      tolerations: {{- toYaml . | nindent 8 }}
      {{- end }}
      {{- if and (not $persistence.enabled) $ephemeral.enabled }}
      volumes:
        - name: data
          emptyDir:
            {{- if $ephemeral.sizeLimit }}
            sizeLimit: {{ $ephemeral.sizeLimit | quote }}
            {{- else }}
            {}
            {{- end }}
      {{- end }}
      {{- if .Values.global.imagePullSecret }}
      imagePullSecrets:
        - name: {{ .Values.global.imagePullSecret }}
      {{- end }}
  {{- if $persistence.enabled }}
  volumeClaimTemplates:
    - metadata:
        name: data
      spec:
        accessModes:
          - {{ default "ReadWriteOnce" $persistence.accessMode | quote }}
        {{- if $persistence.storageClassName }}
        storageClassName: {{ $persistence.storageClassName | quote }}
        {{- end }}
        resources:
          requests:
            storage: {{ default "10Gi" $persistence.size | quote }}
  {{- end }}
{{- end -}}
