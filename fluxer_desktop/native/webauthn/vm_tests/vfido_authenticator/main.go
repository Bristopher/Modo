package main

import (
	"crypto/ecdsa"
	"crypto/elliptic"
	"crypto/rand"
	"crypto/sha256"
	"crypto/x509"
	"crypto/x509/pkix"
	"fmt"
	"io"
	"math/big"
	"os"
	"time"

	virtual_fido "github.com/bulwarkid/virtual-fido"
	"github.com/bulwarkid/virtual-fido/fido_client"
)

type autoApproveSupport struct {
	vaultFilename string
}

func (support *autoApproveSupport) ApproveClientAction(action fido_client.ClientAction, params fido_client.ClientActionRequestParams) bool {
	fmt.Printf("auto-approving action=%d relyingParty=%q user=%q\n", action, params.RelyingParty, params.UserName)
	return true
}

func (support *autoApproveSupport) SaveData(data []byte) {
	err := os.WriteFile(support.vaultFilename, data, 0o600)
	if err != nil {
		panic(fmt.Sprintf("could not write vault: %s", err))
	}
}

func (support *autoApproveSupport) RetrieveData() []byte {
	f, err := os.Open(support.vaultFilename)
	if os.IsNotExist(err) {
		return nil
	}
	if err != nil {
		panic(fmt.Sprintf("could not open vault: %s", err))
	}
	defer f.Close()
	data, err := io.ReadAll(f)
	if err != nil {
		panic(fmt.Sprintf("could not read vault: %s", err))
	}
	return data
}

func (support *autoApproveSupport) Passphrase() string {
	return "vm-test-passphrase"
}

func main() {
	vault := os.Getenv("VFIDO_VAULT")
	if vault == "" {
		vault = "/tmp/vfido-vault.json"
	}
	authority := &x509.Certificate{
		SerialNumber: big.NewInt(0),
		Subject: pkix.Name{
			Organization: []string{"Fluxer VM Test Virtual FIDO"},
			Country:      []string{"US"},
		},
		NotBefore:             time.Now(),
		NotAfter:              time.Now().AddDate(10, 0, 0),
		IsCA:                  true,
		ExtKeyUsage:           []x509.ExtKeyUsage{x509.ExtKeyUsageClientAuth, x509.ExtKeyUsageServerAuth},
		KeyUsage:              x509.KeyUsageDigitalSignature | x509.KeyUsageCertSign,
		BasicConstraintsValid: true,
	}
	privateKey, err := ecdsa.GenerateKey(elliptic.P256(), rand.Reader)
	if err != nil {
		panic(err)
	}
	authorityCertBytes, err := x509.CreateCertificate(rand.Reader, authority, authority, &privateKey.PublicKey, privateKey)
	if err != nil {
		panic(err)
	}
	encryptionKey := sha256.Sum256([]byte("fluxer-vm-test"))

	virtual_fido.SetLogOutput(os.Stdout)
	support := &autoApproveSupport{vaultFilename: vault}
	client := fido_client.NewDefaultClient(authorityCertBytes, privateKey, encryptionKey, support, support)
	fmt.Println("virtual-fido USBIP server starting on 127.0.0.1:3240 (bus 2-2)")
	virtual_fido.Start(client)
}
