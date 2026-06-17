// SPDX-License-Identifier: AGPL-3.0-or-later

import type {IKVProvider} from '@pkgs/kv_client/src/IKVProvider';
import type {ISnowflakeService} from '../../infrastructure/ISnowflakeService';
import {BillingActionIntentRepository} from './BillingActionIntentRepository';
import {BillingChargeRepository} from './BillingChargeRepository';
import {BillingCheckoutSessionRepository} from './BillingCheckoutSessionRepository';
import {BillingCustomerRepository} from './BillingCustomerRepository';
import {BillingDisputeRepository} from './BillingDisputeRepository';
import {BillingInvoiceRepository} from './BillingInvoiceRepository';
import {BillingPaymentIntentRepository} from './BillingPaymentIntentRepository';
import {BillingPaymentMethodRepository} from './BillingPaymentMethodRepository';
import {BillingPaymentRepository} from './BillingPaymentRepository';
import {BillingPriceRepository} from './BillingPriceRepository';
import {BillingProductRepository} from './BillingProductRepository';
import {BillingRefundRepository} from './BillingRefundRepository';
import {BillingSubscriptionRepository} from './BillingSubscriptionRepository';
import {BillingWebhookEventRepository} from './BillingWebhookEventRepository';

export class BillingRepository {
	readonly customers: BillingCustomerRepository;
	readonly products: BillingProductRepository;
	readonly prices: BillingPriceRepository;
	readonly paymentMethods: BillingPaymentMethodRepository;
	readonly subscriptions: BillingSubscriptionRepository;
	readonly invoices: BillingInvoiceRepository;
	readonly paymentIntents: BillingPaymentIntentRepository;
	readonly charges: BillingChargeRepository;
	readonly payments: BillingPaymentRepository;
	readonly refunds: BillingRefundRepository;
	readonly checkoutSessions: BillingCheckoutSessionRepository;
	readonly disputes: BillingDisputeRepository;
	readonly webhookEvents: BillingWebhookEventRepository;
	readonly actionIntents: BillingActionIntentRepository;

	constructor(snowflakeService: ISnowflakeService, kv: IKVProvider) {
		this.payments = new BillingPaymentRepository();
		this.invoices = new BillingInvoiceRepository(this.payments);
		this.customers = new BillingCustomerRepository();
		this.products = new BillingProductRepository();
		this.prices = new BillingPriceRepository();
		this.paymentMethods = new BillingPaymentMethodRepository();
		this.subscriptions = new BillingSubscriptionRepository();
		this.paymentIntents = new BillingPaymentIntentRepository();
		this.charges = new BillingChargeRepository();
		this.refunds = new BillingRefundRepository();
		this.checkoutSessions = new BillingCheckoutSessionRepository();
		this.disputes = new BillingDisputeRepository();
		this.webhookEvents = new BillingWebhookEventRepository(kv);
		this.actionIntents = new BillingActionIntentRepository(snowflakeService);
	}
}
