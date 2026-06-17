{{/* SPDX-License-Identifier: AGPL-3.0-or-later */}}
{{- define "fluxer.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{- define "fluxer.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{- define "fluxer.labels" -}}
helm.sh/chart: {{ include "fluxer.chart" . }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
app.kubernetes.io/part-of: fluxer
{{- end }}

{{- define "fluxer.selectorLabels" -}}
app.kubernetes.io/name: {{ .name }}
app.kubernetes.io/instance: {{ .context.Release.Name }}
{{- end }}

{{- define "fluxer.imagePullSecrets" -}}
imagePullSecrets:
  - name: {{ .Values.global.imagePullSecret }}
{{- end }}

{{- define "fluxer.image" -}}
{{- $tag := required (printf ".tag is required (image: %s)" .image) .tag -}}
{{- $registry := required "global.registry is required" .context.Values.global.registry -}}
{{ $registry }}/{{ .image }}:{{ $tag }}
{{- end }}

{{- define "fluxer.replicas" -}}
{{- $name := .name -}}
{{- $v := .values -}}
{{- $ctx := .context -}}
{{- $desired := int (required (printf ".replicas is required for %s" $name) $v.replicas) -}}
{{- $preserveLiveReplicas := dig "preserveLiveReplicas" true $v -}}
{{- if not $preserveLiveReplicas -}}
{{- $desired -}}
{{- else -}}
{{- $existing := lookup "apps/v1" "Deployment" $ctx.Values.global.namespace $name -}}
{{- if $existing -}}
{{- $current := int (dig "spec" "replicas" 0 $existing) -}}
{{- if gt $current 0 -}}
{{- $current -}}
{{- else -}}
{{- $desired -}}
{{- end -}}
{{- else -}}
{{- $desired -}}
{{- end -}}
{{- end -}}
{{- end }}

{{- define "fluxer.deployment" -}}
{{- $name := .name -}}
{{- $v := .values -}}
{{- $ctx := .context -}}
{{- $isGateway := eq $name "gateway" -}}
{{- $defaultMaxSurge := 1 -}}
{{- $defaultMaxUnavailable := 0 -}}
{{- $defaultMinReadySeconds := 10 -}}
{{- $defaultTerminationGracePeriodSeconds := ternary 90 60 $isGateway -}}
{{- $defaultReadinessPath := ternary "/_health/ready" "/_health" $isGateway -}}
{{- $defaultReadinessTimeoutSeconds := ternary 5 2 $isGateway -}}
{{- $configuredMaxSurge := dig "rollingUpdate" "maxSurge" $defaultMaxSurge $v -}}
{{- $configuredMaxUnavailable := dig "rollingUpdate" "maxUnavailable" $defaultMaxUnavailable $v -}}
{{- $maxSurge := $configuredMaxSurge -}}
{{- $maxUnavailable := $configuredMaxUnavailable -}}
{{- $minReadySeconds := int (dig "minReadySeconds" $defaultMinReadySeconds $v) -}}
{{- $terminationGracePeriodSeconds := int (dig "terminationGracePeriodSeconds" $defaultTerminationGracePeriodSeconds $v) -}}
{{- $readinessPath := dig "readinessProbe" "path" $defaultReadinessPath $v -}}
{{- $readinessExecEnabled := dig "readinessProbe" "execEnabled" $isGateway $v -}}
{{- $readinessTimeoutSeconds := int (dig "readinessProbe" "timeoutSeconds" $defaultReadinessTimeoutSeconds $v) -}}
{{- $readinessExecCommand := printf "curl -fsS --max-time %d http://127.0.0.1:%d%s >/dev/null 2>&1 || exit 1" $readinessTimeoutSeconds (int $v.port) $readinessPath -}}
{{- $readinessInitialDelaySeconds := int (dig "readinessProbe" "initialDelaySeconds" 5 $v) -}}
{{- $readinessPeriodSeconds := int (dig "readinessProbe" "periodSeconds" 5 $v) -}}
{{- $readinessFailureThreshold := int (dig "readinessProbe" "failureThreshold" 2 $v) -}}
{{- $livenessPath := dig "livenessProbe" "path" "/_health" $v -}}
{{- $livenessInitialDelaySeconds := int (dig "livenessProbe" "initialDelaySeconds" 10 $v) -}}
{{- $livenessPeriodSeconds := int (dig "livenessProbe" "periodSeconds" 15 $v) -}}
{{- $livenessFailureThreshold := int (dig "livenessProbe" "failureThreshold" 3 $v) -}}
{{- $livenessTimeoutSeconds := int (dig "livenessProbe" "timeoutSeconds" 5 $v) -}}
{{- $startupProbeEnabled := dig "startupProbe" "enabled" $isGateway $v -}}
{{- $startupProbePath := dig "startupProbe" "path" "/_health" $v -}}
{{- $startupProbeInitialDelaySeconds := int (dig "startupProbe" "initialDelaySeconds" 0 $v) -}}
{{- $startupProbePeriodSeconds := int (dig "startupProbe" "periodSeconds" 5 $v) -}}
{{- $startupProbeFailureThreshold := int (dig "startupProbe" "failureThreshold" 30 $v) -}}
{{- $startupProbeTimeoutSeconds := int (dig "startupProbe" "timeoutSeconds" 5 $v) -}}
{{- $preStopDrainEnabled := dig "preStopDrain" "enabled" $isGateway $v -}}
{{- $preStopDrainPath := dig "preStopDrain" "path" "/_health/drain" $v -}}
{{- $preStopDrainSleepSeconds := int (dig "preStopDrain" "sleepSeconds" 20 $v) -}}
{{- $preStopDrainTimeoutSeconds := int (dig "preStopDrain" "timeoutSeconds" 2 $v) -}}
{{- $preStopDrainRetryCount := int (dig "preStopDrain" "retryCount" 6 $v) -}}
{{- $preStopDrainRetryIntervalSeconds := int (dig "preStopDrain" "retryIntervalSeconds" 1 $v) -}}
{{- $preStopDrainCommand := printf "attempt=0; while [ \"$attempt\" -lt %d ]; do curl -fsS --max-time %d http://127.0.0.1:%d%s >/dev/null 2>&1 && break; attempt=$((attempt+1)); sleep %d; done; sleep %d" $preStopDrainRetryCount $preStopDrainTimeoutSeconds (int $v.port) $preStopDrainPath $preStopDrainRetryIntervalSeconds $preStopDrainSleepSeconds -}}
{{- $build := get $v "build" | default (dict) -}}
{{- $buildVersion := get $build "version" | default $v.tag -}}
{{- $buildSha := get $build "sha" | default "" -}}
{{- $buildChannel := get $build "channel" | default "" -}}
{{- $nsfwServiceEndpoint := get $v "nsfwServiceEndpoint" | default "" -}}
{{- $cluster := get $ctx.Values "cluster" | default (dict) -}}
{{- $gatewayClusterEnabled := and $isGateway (eq (get $cluster "enabled" | default false) true) -}}
{{- $erlangDistribution := get $cluster "erlangDistribution" | default (dict) -}}
{{- $erlangDistPort := int (get $erlangDistribution "port" | default 8081) -}}
{{- $erlangEpmdPort := int (get $erlangDistribution "epmdPort" | default 4369) -}}
{{- $erlangCookieSecret := get $cluster "erlangCookieSecret" | default (dict) -}}
{{- $erlangCookieSecretName := get $erlangCookieSecret "name" | default "fluxer-gateway-erlang-cookie" -}}
{{- $erlangCookieSecretKey := get $erlangCookieSecret "key" | default "cookie" -}}
{{- $gatewayNodeBasename := get $cluster "discoveryNodeBasename" | default "fluxer_gateway" -}}
apiVersion: apps/v1
kind: Deployment
metadata:
  name: {{ $name }}
  namespace: {{ $ctx.Values.global.namespace }}
  labels:
    {{- include "fluxer.labels" $ctx | nindent 4 }}
    {{- include "fluxer.selectorLabels" (dict "name" $name "context" $ctx) | nindent 4 }}
spec:
  replicas: {{ include "fluxer.replicas" (dict "name" $name "values" $v "context" $ctx) }}
  minReadySeconds: {{ $minReadySeconds }}
  selector:
    matchLabels:
      {{- include "fluxer.selectorLabels" (dict "name" $name "context" $ctx) | nindent 6 }}
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: {{ $maxSurge | toJson }}
      maxUnavailable: {{ $maxUnavailable | toJson }}
  template:
    metadata:
      labels:
        {{- include "fluxer.labels" $ctx | nindent 8 }}
        {{- include "fluxer.selectorLabels" (dict "name" $name "context" $ctx) | nindent 8 }}
    spec:
      {{- include "fluxer.imagePullSecrets" $ctx | nindent 6 }}
      terminationGracePeriodSeconds: {{ $terminationGracePeriodSeconds }}
      securityContext:
        runAsNonRoot: true
        seccompProfile:
          type: RuntimeDefault
      {{- if $v.affinity }}
      affinity:
        {{- toYaml $v.affinity | nindent 8 }}
      {{- else if $isGateway }}
      affinity:
        podAntiAffinity:
          preferredDuringSchedulingIgnoredDuringExecution:
            - weight: 100
              podAffinityTerm:
                labelSelector:
                  matchLabels:
                    app.kubernetes.io/name: gateway
                    app.kubernetes.io/instance: {{ $ctx.Release.Name }}
                topologyKey: kubernetes.io/hostname
      {{- end }}
      {{- if $v.topologySpreadConstraints }}
      topologySpreadConstraints:
        {{- toYaml $v.topologySpreadConstraints | nindent 8 }}
      {{- else if $isGateway }}
      topologySpreadConstraints:
        - maxSkew: 1
          topologyKey: kubernetes.io/hostname
          whenUnsatisfiable: ScheduleAnyway
          labelSelector:
            matchLabels:
              app.kubernetes.io/name: gateway
              app.kubernetes.io/instance: {{ $ctx.Release.Name }}
      {{- else }}
      topologySpreadConstraints:
        - maxSkew: 1
          topologyKey: kubernetes.io/hostname
          whenUnsatisfiable: ScheduleAnyway
          nodeAffinityPolicy: Honor
          nodeTaintsPolicy: Honor
          labelSelector:
            matchLabels:
              app.kubernetes.io/name: {{ $name }}
              app.kubernetes.io/instance: {{ $ctx.Release.Name }}
      {{- end }}
      {{- if $v.nodeSelector }}
      nodeSelector:
        {{- toYaml $v.nodeSelector | nindent 8 }}
      {{- end }}
      {{- if $v.tolerations }}
      tolerations:
        {{- toYaml $v.tolerations | nindent 8 }}
      {{- end }}
      containers:
        - name: {{ $name }}
          image: {{ include "fluxer.image" (dict "image" $v.image "tag" $v.tag "context" $ctx) }}
          securityContext:
            allowPrivilegeEscalation: false
            readOnlyRootFilesystem: false
          {{- if $v.command }}
          command: {{ $v.command | toJson }}
          {{- end }}
          ports:
            - name: http
              containerPort: {{ $v.port }}
              protocol: TCP
            {{- if $gatewayClusterEnabled }}
            - name: epmd
              containerPort: {{ $erlangEpmdPort }}
              protocol: TCP
            - name: erl-dist
              containerPort: {{ $erlangDistPort }}
              protocol: TCP
            {{- end }}
          env:
            - name: NODE_ENV
              value: production
            - name: FLUXER_ENV
              value: production
            {{- if $gatewayClusterEnabled }}
            - name: POD_IP
              valueFrom:
                fieldRef:
                  fieldPath: status.podIP
            - name: FLUXER_ERLANG_NODE_NAME
              value: {{ printf "%s@$(POD_IP)" $gatewayNodeBasename | quote }}
            - name: FLUXER_ERLANG_DIST_PORT
              value: {{ printf "%d" $erlangDistPort | quote }}
            - name: FLUXER_ERLANG_COOKIE
              valueFrom:
                secretKeyRef:
                  name: {{ $erlangCookieSecretName }}
                  key: {{ $erlangCookieSecretKey }}
            {{- end }}
            {{- if $buildVersion }}
            - name: BUILD_VERSION
              value: {{ $buildVersion | quote }}
            {{- end }}
            {{- if $buildSha }}
            - name: BUILD_SHA
              value: {{ $buildSha | quote }}
            {{- end }}
            {{- if $buildChannel }}
            - name: RELEASE_CHANNEL
              value: {{ $buildChannel | quote }}
            {{- end }}
            {{- if $nsfwServiceEndpoint }}
            - name: FLUXER_NSFW_SERVICE_ENDPOINT
              value: {{ $nsfwServiceEndpoint | quote }}
            {{- end }}
            {{- if $ctx.Values.global.env }}
            {{- toYaml $ctx.Values.global.env | nindent 12 }}
            {{- end }}
            {{- if $v.env }}
            {{- toYaml $v.env | nindent 12 }}
            {{- end }}
          {{- if or $ctx.Values.global.envFrom $v.envFrom }}
          envFrom:
            {{- if $ctx.Values.global.envFrom }}
            {{- toYaml $ctx.Values.global.envFrom | nindent 12 }}
            {{- end }}
            {{- if $v.envFrom }}
            {{- toYaml $v.envFrom | nindent 12 }}
            {{- end }}
          {{- end }}
          {{- if $preStopDrainEnabled }}
          lifecycle:
            preStop:
              exec:
                command:
                  - /bin/sh
                  - -c
                  - {{ $preStopDrainCommand | quote }}
          {{- end }}
          volumeMounts:
            - name: keys
              mountPath: /etc/fluxer/keys
              readOnly: true
          {{- if not $v.noHealthCheck }}
          livenessProbe:
            httpGet:
              path: {{ $livenessPath | quote }}
              port: http
            initialDelaySeconds: {{ $livenessInitialDelaySeconds }}
            periodSeconds: {{ $livenessPeriodSeconds }}
            timeoutSeconds: {{ $livenessTimeoutSeconds }}
            failureThreshold: {{ $livenessFailureThreshold }}
          readinessProbe:
            {{- if $readinessExecEnabled }}
            exec:
              command:
                - /bin/sh
                - -c
                - {{ $readinessExecCommand | quote }}
            {{- else }}
            httpGet:
              path: {{ $readinessPath | quote }}
              port: http
            {{- end }}
            initialDelaySeconds: {{ $readinessInitialDelaySeconds }}
            periodSeconds: {{ $readinessPeriodSeconds }}
            timeoutSeconds: {{ $readinessTimeoutSeconds }}
            failureThreshold: {{ $readinessFailureThreshold }}
          {{- if $startupProbeEnabled }}
          startupProbe:
            httpGet:
              path: {{ $startupProbePath | quote }}
              port: http
            initialDelaySeconds: {{ $startupProbeInitialDelaySeconds }}
            periodSeconds: {{ $startupProbePeriodSeconds }}
            timeoutSeconds: {{ $startupProbeTimeoutSeconds }}
            failureThreshold: {{ $startupProbeFailureThreshold }}
          {{- end }}
          {{- end }}
          resources:
            {{- toYaml $v.resources | nindent 12 }}
      volumes:
        - name: keys
          secret:
            secretName: fluxer-keys
            optional: true
{{- end }}

{{- define "fluxer.service" -}}
{{- $name := .name -}}
{{- $selectorName := .selectorName | default $name -}}
{{- $v := .values -}}
{{- $ctx := .context -}}
apiVersion: v1
kind: Service
metadata:
  name: {{ $name }}
  namespace: {{ $ctx.Values.global.namespace }}
  labels:
    {{- include "fluxer.labels" $ctx | nindent 4 }}
    {{- include "fluxer.selectorLabels" (dict "name" $name "context" $ctx) | nindent 4 }}
spec:
  type: ClusterIP
  ports:
    - port: {{ $v.port }}
      targetPort: http
      protocol: TCP
      name: http
  selector:
    {{- include "fluxer.selectorLabels" (dict "name" $selectorName "context" $ctx) | nindent 4 }}
{{- end }}

{{- define "fluxer.pdb" -}}
apiVersion: policy/v1
kind: PodDisruptionBudget
metadata:
  name: {{ .name }}-pdb
  namespace: {{ .context.Values.global.namespace }}
  labels:
    {{- include "fluxer.labels" .context | nindent 4 }}
spec:
  minAvailable: {{ .minAvailable }}
  selector:
    matchLabels:
      {{- include "fluxer.selectorLabels" (dict "name" .name "context" .context) | nindent 6 }}
{{- end }}
