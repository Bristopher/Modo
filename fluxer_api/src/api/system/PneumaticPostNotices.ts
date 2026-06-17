// SPDX-License-Identifier: AGPL-3.0-or-later

import {type LocaleCode, Locales} from '@fluxer/constants/src/Locales';

export const PNEUMATIC_POST_SYSTEM_NAME = 'Pneumatic Post';

export const PLUTONIUM_MOBILE_BETA_DISPATCH = {
	key: 'plutonium_mobile_beta_2026_06',
	productName: 'Fluxer',
	premiumProductName: 'Plutonium',
	androidProductName: 'Android',
	iosProductName: 'iOS',
	githubProductName: 'GitHub',
	githubUrl: 'https://github.com/fluxerapp/flutter_client',
	formUrl: 'https://forms.gle/xDZSdJ3LWuYm2wde6',
	testFlightProductName: 'TestFlight',
	appleProductName: 'Apple',
	appStoreProductName: 'Apple App Store',
} as const;

export const PLUTONIUM_MOBILE_BETA_CORRECTION_DISPATCH = {
	key: 'plutonium_mobile_beta_2026_06_mobile_community_correction',
} as const;

const FORM_LINK_LABELS: Partial<Record<LocaleCode, string>> = {
	[Locales.AR]: 'هنا',
	[Locales.BG]: 'тук',
	[Locales.CS]: 'zde',
	[Locales.DA]: 'her',
	[Locales.DE]: 'hier',
	[Locales.EL]: 'εδώ',
	[Locales.EN_GB]: 'here',
	[Locales.EN_US]: 'here',
	[Locales.ES_ES]: 'aquí',
	[Locales.ES_419]: 'aquí',
	[Locales.FI]: 'täällä',
	[Locales.FR]: 'ici',
	[Locales.HE]: 'כאן',
	[Locales.HI]: 'यहाँ',
	[Locales.HR]: 'ovdje',
	[Locales.HU]: 'itt',
	[Locales.ID]: 'di sini',
	[Locales.IT]: 'qui',
	[Locales.JA]: 'こちら',
	[Locales.KO]: '여기',
	[Locales.LT]: 'čia',
	[Locales.NL]: 'hier',
	[Locales.NO]: 'her',
	[Locales.PL]: 'tutaj',
	[Locales.PT_BR]: 'aqui',
	[Locales.RO]: 'aici',
	[Locales.RU]: 'здесь',
	[Locales.SV_SE]: 'här',
	[Locales.TH]: 'ที่นี่',
	[Locales.TR]: 'buradan',
	[Locales.UK]: 'тут',
	[Locales.VI]: 'tại đây',
	[Locales.ZH_CN]: '这里',
	[Locales.ZH_TW]: '此處',
};

function markdownFormLink(label: string): string {
	return `[${label}](${PLUTONIUM_MOBILE_BETA_DISPATCH.formUrl})`;
}

function getFormLink(locale: LocaleCode | string | null | undefined): string {
	return markdownFormLink(FORM_LINK_LABELS[locale as LocaleCode] ?? 'here');
}

function buildTokenValues(locale: LocaleCode | string | null | undefined, userId: string): Record<string, string> {
	return {
		product_name: PLUTONIUM_MOBILE_BETA_DISPATCH.productName,
		premium_name: PLUTONIUM_MOBILE_BETA_DISPATCH.premiumProductName,
		android_name: PLUTONIUM_MOBILE_BETA_DISPATCH.androidProductName,
		ios_name: PLUTONIUM_MOBILE_BETA_DISPATCH.iosProductName,
		github_name: PLUTONIUM_MOBILE_BETA_DISPATCH.githubProductName,
		github_link: `[${PLUTONIUM_MOBILE_BETA_DISPATCH.githubProductName}](${PLUTONIUM_MOBILE_BETA_DISPATCH.githubUrl})`,
		form_link: getFormLink(locale),
		form_link_here: markdownFormLink('here'),
		form_link_ar: markdownFormLink('هنا'),
		form_link_bg: markdownFormLink('тук'),
		form_link_cs: markdownFormLink('zde'),
		form_link_da: markdownFormLink('her'),
		form_link_de: markdownFormLink('hier'),
		form_link_el: markdownFormLink('εδώ'),
		form_link_es: markdownFormLink('aquí'),
		form_link_fi: markdownFormLink('täällä'),
		form_link_fr: markdownFormLink('ici'),
		form_link_he: markdownFormLink('כאן'),
		form_link_hi: markdownFormLink('यहाँ'),
		form_link_hr: markdownFormLink('ovdje'),
		form_link_hu: markdownFormLink('itt'),
		form_link_id: markdownFormLink('di sini'),
		form_link_it: markdownFormLink('qui'),
		form_link_ja: markdownFormLink('こちら'),
		form_link_ko: markdownFormLink('여기'),
		form_link_lt: markdownFormLink('čia'),
		form_link_nl: markdownFormLink('hier'),
		form_link_no: markdownFormLink('her'),
		form_link_pl: markdownFormLink('tutaj'),
		form_link_pt_br: markdownFormLink('aqui'),
		form_link_ro: markdownFormLink('aici'),
		form_link_ru: markdownFormLink('здесь'),
		form_link_sv: markdownFormLink('här'),
		form_link_th: markdownFormLink('ที่นี่'),
		form_link_tr: markdownFormLink('buradan'),
		form_link_uk: markdownFormLink('тут'),
		form_link_vi: markdownFormLink('tại đây'),
		form_link_zh_cn: markdownFormLink('这里'),
		form_link_zh_tw: markdownFormLink('此處'),
		testflight_name: PLUTONIUM_MOBILE_BETA_DISPATCH.testFlightProductName,
		apple_name: PLUTONIUM_MOBILE_BETA_DISPATCH.appleProductName,
		app_store_name: PLUTONIUM_MOBILE_BETA_DISPATCH.appStoreProductName,
		user_id: userId,
	};
}

const ENGLISH_TEMPLATE = `Calling all {{premium_name}} subscribers!

Thank you for helping make {{product_name}} happen. We are excited to announce the launch of our mobile app beta. The beta is open to all {{android_name}} users and to {{premium_name}} members on {{ios_name}}.

Our mobile app will become available on 15 June 2026. When we launch, {{android_name}} users will be able to download the APK from {{github_link}}, and {{ios_name}} users will be able to access it through {{apple_name}}'s {{testflight_name}}.

If you are an {{ios_name}} user, please submit your {{app_store_name}} email address {{form_link_here}}. The app will only work with your account during the beta.

{{premium_name}} members will also be able to join our mobile community. During the beta, this will be our hub for bugs, feature requests, UX changes, and everything in between. If you would like to be added, please let us know in the form above.

As a reminder, this is beta software. Many features will not be present yet, and you should expect bugs. Your feedback and help in finding bugs will help us improve quickly.

Thank you once again,
The {{product_name}} Team`;

const LOCALIZED_TEMPLATES: Partial<Record<LocaleCode, string>> = {
	[Locales.AR]: `نداء إلى جميع مشتركي {{premium_name}}!

شكرًا لكم على المساعدة في جعل {{product_name}} واقعًا. يسعدنا الإعلان عن إطلاق النسخة التجريبية من تطبيقنا للهواتف المحمولة. النسخة التجريبية متاحة لجميع مستخدمي {{android_name}} ولمشتركي {{premium_name}} على {{ios_name}}.

سيصبح تطبيقنا للهواتف المحمولة متاحًا في 15 يونيو 2026. عند الإطلاق، سيتمكن مستخدمو {{android_name}} من تنزيل ملف APK من {{github_link}}، وسيتمكن مستخدمو {{ios_name}} من الوصول إليه عبر {{testflight_name}} من {{apple_name}}.

إذا كنت من مستخدمي {{ios_name}}، يُرجى إرسال عنوان بريدك الإلكتروني المستخدم في {{app_store_name}} {{form_link_ar}}. سيعمل التطبيق فقط مع حسابك خلال فترة النسخة التجريبية.

سيتمكن أعضاء {{premium_name}} أيضًا من الانضمام إلى مجتمعنا الخاص بتطبيق الهاتف المحمول. خلال فترة النسخة التجريبية، سيكون هذا المجتمع مركزنا للإبلاغ عن الأخطاء وطلبات الميزات وتغييرات تجربة المستخدم وكل ما يتعلق بذلك. إذا كنت ترغب في إضافتك، فيُرجى إعلامنا عبر النموذج أعلاه.

نذكّركم بأن هذا البرنامج لا يزال في مرحلة تجريبية. لن تتوفر العديد من الميزات بعد، ويجب أن تتوقعوا وجود أخطاء. ستساعدنا ملاحظاتكم ومساعدتكم في العثور على الأخطاء على التحسين بسرعة.

شكرًا لكم مرة أخرى،
فريق {{product_name}}`,
	[Locales.BG]: `До всички абонати на {{premium_name}}!

Благодарим ви, че помогнахте {{product_name}} да стане реалност. Радваме се да обявим старта на бета версията на нашето мобилно приложение. Бета версията е отворена за всички потребители на {{android_name}} и за членовете на {{premium_name}} на {{ios_name}}.

Мобилното ни приложение ще бъде достъпно на 15 юни 2026 г. При старта потребителите на {{android_name}} ще могат да изтеглят APK файла от {{github_link}}, а потребителите на {{ios_name}} ще имат достъп до приложението чрез {{testflight_name}} на {{apple_name}}.

Ако използвате {{ios_name}}, моля, изпратете имейл адреса, който използвате за {{app_store_name}}, {{form_link_bg}}. По време на бетата приложението ще работи само с вашия акаунт.

Членовете на {{premium_name}} също ще могат да се присъединят към нашата мобилна общност. По време на бетата тя ще бъде мястото за съобщаване на бъгове, заявки за нови функции, промени в потребителското изживяване и всичко между тях. Ако искате да бъдете добавени, моля, уведомете ни чрез формуляра по-горе.

Напомняме, че това е бета софтуер. Много функции все още няма да са налични и трябва да очаквате бъгове. Вашата обратна връзка и помощта ви при намирането на бъгове ще ни помогнат да подобряваме приложението бързо.

Още веднъж ви благодарим,
Екипът на {{product_name}}`,
	[Locales.CS]: `Voláme všechny předplatitele {{premium_name}}!

Děkujeme, že pomáháte uskutečnit {{product_name}}. S radostí oznamujeme spuštění beta verze naší mobilní aplikace. Beta je otevřená všem uživatelům systému {{android_name}} a členům {{premium_name}} na {{ios_name}}.

Naše mobilní aplikace bude dostupná 15. června 2026. Po spuštění si uživatelé systému {{android_name}} budou moci stáhnout APK z {{github_link}} a uživatelé {{ios_name}} k ní získají přístup přes {{testflight_name}} od {{apple_name}}.

Pokud používáte {{ios_name}}, odešlete prosím svou e-mailovou adresu používanou pro {{app_store_name}} {{form_link_cs}}. Během bety bude aplikace fungovat pouze s vaším účtem.

Členové {{premium_name}} se také budou moci připojit k naší mobilní komunitě. Během bety bude sloužit jako naše centrum pro hlášení chyb, požadavky na funkce, změny v uživatelském prostředí a vše ostatní. Pokud chcete být přidáni, dejte nám prosím vědět ve výše uvedeném formuláři.

Připomínáme, že jde o beta software. Mnoho funkcí zatím nebude k dispozici a je třeba počítat s chybami. Vaše zpětná vazba a pomoc s jejich hledáním nám pomohou rychle se zlepšovat.

Ještě jednou děkujeme,
Tým {{product_name}}`,
	[Locales.DA]: `Til alle {{premium_name}}-abonnenter!

Tak, fordi I hjælper med at gøre {{product_name}} til virkelighed. Vi er glade for at kunne annoncere lanceringen af betaen til vores mobilapp. Betaen er åben for alle {{android_name}}-brugere og for {{premium_name}}-medlemmer på {{ios_name}}.

Vores mobilapp bliver tilgængelig den 15. juni 2026. Ved lanceringen vil {{android_name}}-brugere kunne downloade APK'en fra {{github_link}}, og {{ios_name}}-brugere vil kunne få adgang til den via {{testflight_name}} fra {{apple_name}}.

Hvis du bruger {{ios_name}}, bedes du indsende den e-mailadresse, du bruger til {{app_store_name}}, {{form_link_da}}. Appen vil kun fungere med din konto under betaen.

{{premium_name}}-medlemmer vil også kunne deltage i vores mobilfællesskab. Under betaen bliver det vores samlingspunkt for fejl, funktionsønsker, ændringer til brugeroplevelsen og alt derimellem. Hvis du gerne vil tilføjes, så giv os besked i formularen ovenfor.

Som en påmindelse er dette betasoftware. Mange funktioner vil endnu ikke være tilgængelige, og du skal forvente fejl. Din feedback og hjælp med at finde fejl vil hjælpe os med at forbedre appen hurtigt.

Endnu en gang tak,
{{product_name}}-teamet`,
	[Locales.DE]: `Aufruf an alle {{premium_name}}-Abonnenten!

Vielen Dank, dass ihr dabei helft, {{product_name}} möglich zu machen. Wir freuen uns, den Start der Beta unserer mobilen App anzukündigen. Die Beta steht allen {{android_name}}-Nutzern sowie {{premium_name}}-Mitgliedern auf {{ios_name}} offen.

Unsere mobile App wird ab dem 15. Juni 2026 verfügbar sein. Zum Start können {{android_name}}-Nutzer die APK über {{github_link}} herunterladen, während {{ios_name}}-Nutzer über {{testflight_name}} von {{apple_name}} Zugriff erhalten.

Wenn du {{ios_name}} nutzt, reiche bitte {{form_link_de}} die E-Mail-Adresse ein, die du für den {{app_store_name}} verwendest. Während der Beta funktioniert die App nur mit deinem Konto.

{{premium_name}}-Mitglieder können außerdem unserer mobilen Community beitreten. Während der Beta ist sie unsere zentrale Anlaufstelle für Bugs, Feature-Wünsche, Änderungen an der Benutzererfahrung und alles dazwischen. Wenn du hinzugefügt werden möchtest, lass es uns bitte im oben genannten Formular wissen.

Zur Erinnerung: Es handelt sich um Beta-Software. Viele Funktionen werden noch nicht verfügbar sein, und du solltest mit Fehlern rechnen. Dein Feedback und deine Hilfe beim Finden von Bugs werden uns dabei helfen, uns schnell zu verbessern.

Nochmals vielen Dank,
Das {{product_name}}-Team`,
	[Locales.EL]: `Κάλεσμα σε όλους τους συνδρομητές {{premium_name}}!

Σας ευχαριστούμε που βοηθήσατε να γίνει το {{product_name}} πραγματικότητα. Με χαρά ανακοινώνουμε την κυκλοφορία της beta έκδοσης της εφαρμογής μας για κινητά. Η beta είναι διαθέσιμη σε όλους τους χρήστες {{android_name}} και στα μέλη {{premium_name}} στο {{ios_name}}.

Η εφαρμογή μας για κινητά θα γίνει διαθέσιμη στις 15 Ιουνίου 2026. Με την κυκλοφορία της, οι χρήστες {{android_name}} θα μπορούν να κατεβάσουν το APK από το {{github_link}}, ενώ οι χρήστες {{ios_name}} θα μπορούν να αποκτήσουν πρόσβαση μέσω του {{testflight_name}} της {{apple_name}}.

Αν είστε χρήστης {{ios_name}}, παρακαλούμε υποβάλετε τη διεύθυνση email που χρησιμοποιείτε για το {{app_store_name}} {{form_link_el}}. Κατά τη διάρκεια της beta, η εφαρμογή θα λειτουργεί μόνο με τον λογαριασμό σας.

Τα μέλη {{premium_name}} θα μπορούν επίσης να συμμετάσχουν στην κοινότητά μας για την εφαρμογή για κινητά. Κατά τη διάρκεια της beta, αυτή θα είναι ο κεντρικός μας χώρος για σφάλματα, αιτήματα λειτουργιών, αλλαγές στην εμπειρία χρήστη και οτιδήποτε άλλο. Αν θέλετε να προστεθείτε, ενημερώστε μας μέσω της παραπάνω φόρμας.

Υπενθυμίζουμε ότι πρόκειται για beta λογισμικό. Πολλές λειτουργίες δεν θα είναι ακόμη διαθέσιμες και θα πρέπει να περιμένετε σφάλματα. Τα σχόλιά σας και η βοήθειά σας στον εντοπισμό σφαλμάτων θα μας βοηθήσουν να βελτιωθούμε γρήγορα.

Σας ευχαριστούμε ξανά,
Η ομάδα του {{product_name}}`,
	[Locales.ES_ES]: `¡Llamamiento a todos los suscriptores de {{premium_name}}!

Gracias por ayudar a hacer realidad {{product_name}}. Nos alegra anunciar el lanzamiento de la beta de nuestra aplicación móvil. La beta está abierta a todos los usuarios de {{android_name}} y a los miembros de {{premium_name}} en {{ios_name}}.

Nuestra aplicación móvil estará disponible el 15 de junio de 2026. En el lanzamiento, los usuarios de {{android_name}} podrán descargar el APK desde {{github_link}}, y los usuarios de {{ios_name}} podrán acceder a ella a través de {{testflight_name}} de {{apple_name}}.

Si usas {{ios_name}}, envía la dirección de correo electrónico que utilizas para el {{app_store_name}} {{form_link_es}}. Durante la beta, la aplicación solo funcionará con tu cuenta.

Los miembros de {{premium_name}} también podrán unirse a nuestra comunidad móvil. Durante la beta, será nuestro punto de encuentro para errores, solicitudes de funciones, cambios de experiencia de usuario y todo lo demás. Si quieres que te añadamos, indícanoslo en el formulario anterior.

Como recordatorio, se trata de software en fase beta. Muchas funciones todavía no estarán disponibles y debes esperar que haya errores. Tus comentarios y tu ayuda para encontrar errores nos ayudarán a mejorar rápidamente.

Gracias de nuevo,
El equipo de {{product_name}}`,
	[Locales.ES_419]: `¡Atención, suscriptores de {{premium_name}}!

Gracias por ayudar a hacer realidad {{product_name}}. Nos emociona anunciar el lanzamiento de la beta de nuestra app móvil. La beta está abierta a todos los usuarios de {{android_name}} y a los miembros de {{premium_name}} en {{ios_name}}.

Nuestra app móvil estará disponible el 15 de junio de 2026. En el lanzamiento, los usuarios de {{android_name}} podrán descargar el APK desde {{github_link}}, y los usuarios de {{ios_name}} podrán acceder a ella a través de {{testflight_name}} de {{apple_name}}.

Si usas {{ios_name}}, envía la dirección de correo electrónico que usas para {{app_store_name}} {{form_link_es}}. Durante la beta, la app solo funcionará con tu cuenta.

Los miembros de {{premium_name}} también podrán unirse a nuestra comunidad móvil. Durante la beta, será nuestro espacio central para reportar errores, solicitar funciones, proponer cambios en la experiencia de usuario y todo lo demás. Si quieres que te agreguemos, avísanos en el formulario anterior.

Recuerda que este es software beta. Muchas funciones todavía no estarán disponibles y es posible que encuentres errores. Tus comentarios y tu ayuda para encontrarlos nos ayudarán a mejorar rápidamente.

Gracias de nuevo,
El equipo de {{product_name}}`,
	[Locales.FI]: `Huomio kaikki {{premium_name}}-tilaajat!

Kiitos, että olette auttaneet tekemään {{product_name}}-palvelusta totta. Olemme iloisia voidessamme ilmoittaa mobiilisovelluksemme betaversion julkaisusta. Beta on avoin kaikille {{android_name}}-käyttäjille sekä {{premium_name}}-jäsenille {{ios_name}}:ssä.

Mobiilisovelluksemme tulee saataville 15. kesäkuuta 2026. Julkaisun yhteydessä {{android_name}}-käyttäjät voivat ladata APK:n osoitteesta {{github_link}}, ja {{ios_name}}-käyttäjät pääsevät käyttämään sovellusta {{apple_name}}-yhtiön {{testflight_name}}-palvelun kautta.

Jos käytät {{ios_name}}:ää, lähetä palvelussa {{app_store_name}} käyttämäsi sähköpostiosoite {{form_link_fi}}. Betan aikana sovellus toimii vain omalla tililläsi.

{{premium_name}}-jäsenet voivat myös liittyä mobiiliyhteisöömme. Betan aikana se toimii keskuksenamme bugeille, ominaisuuspyynnöille, käyttökokemukseen liittyville muutoksille ja kaikelle siltä väliltä. Jos haluat tulla lisätyksi, ilmoita siitä meille yllä olevalla lomakkeella.

Muistutuksena: kyseessä on betaohjelmisto. Monet ominaisuudet eivät ole vielä saatavilla, ja bugeja on odotettavissa. Palautteesi ja apusi bugien löytämisessä auttavat meitä parantamaan sovellusta nopeasti.

Kiitos vielä kerran,
{{product_name}}-tiimi`,
	[Locales.FR]: `À tous les abonnés {{premium_name}} !

Merci de nous aider à rendre {{product_name}} possible. Nous sommes ravis d’annoncer le lancement de la bêta de notre application mobile. La bêta est ouverte à tous les utilisateurs {{android_name}} ainsi qu’aux membres {{premium_name}} sur {{ios_name}}.

Notre application mobile sera disponible le 15 juin 2026. Au lancement, les utilisateurs {{android_name}} pourront télécharger l’APK depuis {{github_link}}, et les utilisateurs {{ios_name}} pourront y accéder via {{testflight_name}} d’{{apple_name}}.

Si vous utilisez {{ios_name}}, veuillez envoyer l’adresse e-mail que vous utilisez pour l’{{app_store_name}} {{form_link_fr}}. Pendant la bêta, l’application ne fonctionnera qu’avec votre compte.

Les membres {{premium_name}} pourront également rejoindre notre communauté mobile. Pendant la bêta, ce sera notre espace central pour les bugs, les demandes de fonctionnalités, les changements liés à l’expérience utilisateur et tout le reste. Si vous souhaitez être ajouté, faites-le-nous savoir dans le formulaire ci-dessus.

Pour rappel, il s’agit d’un logiciel en version bêta. De nombreuses fonctionnalités ne seront pas encore présentes, et vous devez vous attendre à rencontrer des bugs. Vos retours et votre aide pour trouver les bugs nous aideront à nous améliorer rapidement.

Encore merci,
L’équipe {{product_name}}`,
	[Locales.HE]: `לכל מנויי {{premium_name}}!

תודה שאתם עוזרים להפוך את {{product_name}} למציאות. אנחנו שמחים להכריז על השקת גרסת הבטא של אפליקציית המובייל שלנו. הבטא פתוחה לכל משתמשי {{android_name}} ולחברי {{premium_name}} ב-{{ios_name}}.

אפליקציית המובייל שלנו תהיה זמינה ב-15 ביוני 2026. עם ההשקה, משתמשי {{android_name}} יוכלו להוריד את קובץ ה-APK מ-{{github_link}}, ומשתמשי {{ios_name}} יוכלו לגשת אליה דרך {{testflight_name}} של {{apple_name}}.

אם אתם משתמשים ב-{{ios_name}}, אנא שלחו את כתובת הדוא״ל שבה אתם משתמשים עבור {{app_store_name}} {{form_link_he}}. במהלך הבטא, האפליקציה תפעל רק עם החשבון שלכם.

חברי {{premium_name}} יוכלו גם להצטרף לקהילת המובייל שלנו. במהלך הבטא, היא תהיה המרכז שלנו לדיווח על באגים, בקשות לפיצ׳רים, שינויים בחוויית המשתמש וכל מה שביניהם. אם תרצו שנוסיף אתכם, אנא הודיעו לנו בטופס שלמעלה.

נזכיר שמדובר בתוכנת בטא. תכונות רבות עדיין לא יהיו זמינות, ויש לצפות לבאגים. המשוב שלכם והעזרה שלכם באיתור באגים יעזרו לנו להשתפר במהירות.

שוב תודה,
צוות {{product_name}}`,
	[Locales.HI]: `{{premium_name}} के सभी सब्सक्राइबर ध्यान दें!

{{product_name}} को संभव बनाने में मदद करने के लिए धन्यवाद। हमें अपने मोबाइल ऐप बीटा के लॉन्च की घोषणा करते हुए खुशी हो रही है। बीटा सभी {{android_name}} उपयोगकर्ताओं और {{ios_name}} पर {{premium_name}} सदस्यों के लिए उपलब्ध है।

हमारा मोबाइल ऐप 15 जून 2026 को उपलब्ध होगा। लॉन्च के समय, {{android_name}} उपयोगकर्ता {{github_link}} से APK डाउनलोड कर सकेंगे, और {{ios_name}} उपयोगकर्ता {{apple_name}} के {{testflight_name}} के ज़रिए ऐप एक्सेस कर सकेंगे।

यदि आप {{ios_name}} उपयोगकर्ता हैं, तो कृपया अपना {{app_store_name}} ईमेल पता {{form_link_hi}} सबमिट करें। बीटा के दौरान ऐप केवल आपके खाते के साथ ही काम करेगा।

{{premium_name}} सदस्य हमारे मोबाइल समुदाय में भी शामिल हो सकेंगे। बीटा के दौरान, यह बग्स, फीचर अनुरोधों, UX बदलावों और बाकी सभी बातों के लिए हमारा मुख्य केंद्र होगा। यदि आप जोड़े जाना चाहते हैं, तो कृपया ऊपर दिए गए फ़ॉर्म में हमें बताएं।

याद रखें, यह बीटा सॉफ़्टवेयर है। कई फीचर्स अभी उपलब्ध नहीं होंगे, और आपको बग्स मिलने की उम्मीद रखनी चाहिए। आपका फ़ीडबैक और बग्स खोजने में आपकी मदद हमें तेज़ी से सुधार करने में मदद करेगी।

एक बार फिर धन्यवाद,
{{product_name}} टीम`,
	[Locales.HR]: `Poziv svim pretplatnicima programa {{premium_name}}!

Hvala vam što pomažete da {{product_name}} postane stvarnost. S veseljem najavljujemo pokretanje beta verzije naše mobilne aplikacije. Beta je otvorena za sve korisnike sustava {{android_name}} i za članove programa {{premium_name}} na platformi {{ios_name}}.

Naša mobilna aplikacija bit će dostupna 15. lipnja 2026. Pri pokretanju će korisnici sustava {{android_name}} moći preuzeti APK s {{github_link}}, a korisnici platforme {{ios_name}} moći će joj pristupiti putem servisa {{testflight_name}} od {{apple_name}}.

Ako ste korisnik platforme {{ios_name}}, pošaljite svoju adresu e-pošte za {{app_store_name}} {{form_link_hr}}. Tijekom bete aplikacija će raditi samo s vašim računom.

Članovi programa {{premium_name}} također će se moći pridružiti našoj mobilnoj zajednici. Tijekom bete to će biti naše središnje mjesto za bugove, zahtjeve za značajke, promjene korisničkog iskustva i sve ostalo. Ako želite biti dodani, obavijestite nas putem gore navedenog obrasca.

Podsjećamo, ovo je beta softver. Mnoge značajke još neće biti dostupne i trebate očekivati bugove. Vaše povratne informacije i pomoć u pronalaženju bugova pomoći će nam da se brzo poboljšavamo.

Još jednom hvala,
{{product_name}} tim`,
	[Locales.HU]: `Figyelem, {{premium_name}}-előfizetők!

Köszönjük, hogy segítetek abban, hogy a {{product_name}} valósággá váljon. Örömmel jelentjük be mobilalkalmazásunk bétaverziójának indulását. A béta minden {{android_name}}-felhasználó, valamint az {{ios_name}}-t használó {{premium_name}}-tagok számára elérhető.

Mobilalkalmazásunk 2026. június 15-én válik elérhetővé. Az induláskor az {{android_name}}-felhasználók innen tölthetik le az APK-t: {{github_link}}, az {{ios_name}}-felhasználók pedig a {{testflight_name}} szolgáltatáson keresztül férhetnek hozzá az {{apple_name}} jóvoltából.

Ha {{ios_name}}-t használsz, kérjük, add meg az {{app_store_name}} szolgáltatásban használt e-mail-címedet {{form_link_hu}}. A béta ideje alatt az alkalmazás csak a te fiókoddal fog működni.

A {{premium_name}}-tagok csatlakozhatnak mobilos közösségünkhöz is. A béta ideje alatt ez lesz a központi helyünk a hibák, funkciókérések, UX-módosítások és minden egyéb téma számára. Ha szeretnéd, hogy hozzáadjunk, jelezd a fenti űrlapon.

Emlékeztetőül: ez bétaszoftver. Sok funkció még nem lesz elérhető, és hibákra is számítani kell. A visszajelzéseitek és a hibák megtalálásában nyújtott segítségetek segít nekünk gyorsan fejlődni.

Még egyszer köszönjük,
A {{product_name}} csapata`,
	[Locales.ID]: `Untuk semua pelanggan {{premium_name}}!

Terima kasih telah membantu mewujudkan {{product_name}}. Kami senang mengumumkan peluncuran beta aplikasi seluler kami. Beta ini terbuka untuk semua pengguna {{android_name}} dan anggota {{premium_name}} di {{ios_name}}.

Aplikasi seluler kami akan tersedia pada 15 Juni 2026. Saat peluncuran, pengguna {{android_name}} dapat mengunduh APK dari {{github_link}}, dan pengguna {{ios_name}} dapat mengaksesnya melalui {{testflight_name}} dari {{apple_name}}.

Jika Anda pengguna {{ios_name}}, harap kirimkan alamat email {{app_store_name}} Anda {{form_link_id}}. Selama beta, aplikasi hanya akan berfungsi dengan akun Anda.

Anggota {{premium_name}} juga dapat bergabung dengan komunitas seluler kami. Selama beta, komunitas ini akan menjadi pusat kami untuk bug, permintaan fitur, perubahan UX, dan berbagai hal lainnya. Jika Anda ingin ditambahkan, beri tahu kami melalui formulir di atas.

Sebagai pengingat, ini adalah perangkat lunak beta. Banyak fitur belum akan tersedia, dan Anda sebaiknya mengantisipasi adanya bug. Masukan Anda dan bantuan Anda dalam menemukan bug akan membantu kami meningkatkan aplikasi dengan cepat.

Terima kasih sekali lagi,
Tim {{product_name}}`,
	[Locales.IT]: `A tutti gli abbonati {{premium_name}}!

Grazie per aver contribuito a rendere {{product_name}} realtà. Siamo felici di annunciare il lancio della beta della nostra app mobile. La beta è aperta a tutti gli utenti {{android_name}} e ai membri {{premium_name}} su {{ios_name}}.

La nostra app mobile sarà disponibile dal 15 giugno 2026. Al lancio, gli utenti {{android_name}} potranno scaricare l’APK da {{github_link}}, mentre gli utenti {{ios_name}} potranno accedervi tramite {{testflight_name}} di {{apple_name}}.

Se usi {{ios_name}}, invia l’indirizzo email che utilizzi per l’{{app_store_name}} {{form_link_it}}. Durante la beta, l’app funzionerà solo con il tuo account.

I membri {{premium_name}} potranno anche unirsi alla nostra community mobile. Durante la beta, sarà il nostro punto di riferimento per bug, richieste di funzionalità, modifiche all’esperienza utente e tutto il resto. Se vuoi essere aggiunto, faccelo sapere nel modulo qui sopra.

Ricordiamo che si tratta di software beta. Molte funzionalità non saranno ancora disponibili e dovresti aspettarti la presenza di bug. Il tuo feedback e il tuo aiuto nel trovare bug ci aiuteranno a migliorare rapidamente.

Grazie ancora,
Il team {{product_name}}`,
	[Locales.JA]: `{{premium_name}}サブスクライバーの皆さまへ

{{product_name}}の実現にご協力いただき、ありがとうございます。モバイルアプリのベータ版をリリースすることをお知らせします。ベータ版は、すべての{{android_name}}ユーザーと、{{ios_name}}をご利用の{{premium_name}}メンバーが対象です。

モバイルアプリは2026年6月15日に利用可能になります。リリース時には、{{android_name}}ユーザーは{{github_link}}からAPKをダウンロードでき、{{ios_name}}ユーザーは{{apple_name}}の{{testflight_name}}を通じてアクセスできます。

{{ios_name}}をご利用の方は、{{app_store_name}}で使用しているメールアドレスを{{form_link_ja}}からご送信ください。ベータ期間中、アプリはお客様のアカウントでのみ動作します。

{{premium_name}}メンバーは、モバイルコミュニティにも参加できます。ベータ期間中、このコミュニティはバグ、機能リクエスト、UX変更、その他あらゆるフィードバックの窓口となります。追加を希望される場合は、上記のフォームでお知らせください。

なお、これはベータ版ソフトウェアです。まだ多くの機能は搭載されておらず、不具合が発生する可能性があります。皆さまからのフィードバックやバグ発見へのご協力は、迅速な改善に大きく役立ちます。

改めてありがとうございます。
{{product_name}}チーム`,
	[Locales.KO]: `{{premium_name}} 구독자 여러분께!

{{product_name}}를 실현할 수 있도록 도와주셔서 감사합니다. 모바일 앱 베타 출시 소식을 전하게 되어 기쁩니다. 베타는 모든 {{android_name}} 사용자와 {{ios_name}}의 {{premium_name}} 회원에게 열려 있습니다.

모바일 앱은 2026년 6월 15일에 제공될 예정입니다. 출시 후 {{android_name}} 사용자는 {{github_link}}에서 APK를 다운로드할 수 있으며, {{ios_name}} 사용자는 {{apple_name}}의 {{testflight_name}}를 통해 이용할 수 있습니다.

{{ios_name}} 사용자라면 {{app_store_name}}에서 사용하는 이메일 주소를 {{form_link_ko}}에 제출해 주세요. 베타 기간 동안 앱은 제출하신 계정에서만 작동합니다.

{{premium_name}} 회원은 모바일 커뮤니티에도 참여할 수 있습니다. 베타 기간 동안 이곳은 버그, 기능 요청, UX 변경 사항 등 모든 의견을 다루는 중심 공간이 될 것입니다. 추가를 원하시면 위 양식을 통해 알려 주세요.

다시 알려드리지만, 이는 베타 소프트웨어입니다. 아직 많은 기능이 포함되지 않았으며, 버그가 있을 수 있습니다. 여러분의 피드백과 버그 발견에 대한 도움은 저희가 빠르게 개선하는 데 큰 도움이 됩니다.

다시 한 번 감사드립니다.
{{product_name}} 팀`,
	[Locales.LT]: `Kviečiame visus {{premium_name}} prenumeratorius!

Dėkojame, kad padedate paversti {{product_name}} realybe. Džiaugiamės galėdami pranešti apie mūsų mobiliosios programėlės beta versijos paleidimą. Beta versija atvira visiems {{android_name}} naudotojams ir {{premium_name}} nariams, naudojantiems {{ios_name}}.

Mūsų mobilioji programėlė bus pasiekiama 2026 m. birželio 15 d. Paleidimo metu {{android_name}} naudotojai galės atsisiųsti APK iš {{github_link}}, o {{ios_name}} naudotojai galės ją pasiekti per {{apple_name}} {{testflight_name}}.

Jei naudojate {{ios_name}}, pateikite savo {{app_store_name}} el. pašto adresą {{form_link_lt}}. Beta laikotarpiu programėlė veiks tik su jūsų paskyra.

{{premium_name}} nariai taip pat galės prisijungti prie mūsų mobiliosios bendruomenės. Beta laikotarpiu ji bus pagrindinė vieta pranešti apie klaidas, teikti funkcijų prašymus, siūlyti UX pakeitimus ir aptarti visa kita. Jei norėtumėte būti pridėti, praneškite mums aukščiau pateiktoje formoje.

Primename, kad tai yra beta programinė įranga. Daugelio funkcijų dar nebus, todėl reikėtų tikėtis klaidų. Jūsų atsiliepimai ir pagalba ieškant klaidų padės mums greitai tobulėti.

Dar kartą dėkojame,
{{product_name}} komanda`,
	[Locales.NL]: `Aan alle {{premium_name}}-abonnees!

Bedankt dat jullie helpen om {{product_name}} mogelijk te maken. We zijn blij de lancering van de bèta van onze mobiele app aan te kondigen. De bèta staat open voor alle {{android_name}}-gebruikers en voor {{premium_name}}-leden op {{ios_name}}.

Onze mobiele app komt beschikbaar op 15 juni 2026. Bij de lancering kunnen {{android_name}}-gebruikers de APK downloaden via {{github_link}}, en {{ios_name}}-gebruikers krijgen toegang via {{testflight_name}} van {{apple_name}}.

Als je {{ios_name}} gebruikt, dien dan {{form_link_nl}} het e-mailadres in dat je voor de {{app_store_name}} gebruikt. Tijdens de bèta werkt de app alleen met jouw account.

{{premium_name}}-leden kunnen ook lid worden van onze mobiele community. Tijdens de bèta is dit onze centrale plek voor bugs, functieverzoeken, UX-wijzigingen en alles daartussenin. Als je wilt worden toegevoegd, laat het ons weten via het bovenstaande formulier.

Ter herinnering: dit is bètasoftware. Veel functies zullen nog niet aanwezig zijn en je moet rekening houden met bugs. Jullie feedback en hulp bij het vinden van bugs helpen ons om snel te verbeteren.

Nogmaals bedankt,
Het {{product_name}}-team`,
	[Locales.NO]: `Til alle {{premium_name}}-abonnenter!

Takk for at dere hjelper med å gjøre {{product_name}} mulig. Vi er glade for å kunne kunngjøre lanseringen av betaen for mobilappen vår. Betaen er åpen for alle {{android_name}}-brukere og for {{premium_name}}-medlemmer på {{ios_name}}.

Mobilappen vår blir tilgjengelig 15. juni 2026. Ved lansering vil {{android_name}}-brukere kunne laste ned APK-en fra {{github_link}}, og {{ios_name}}-brukere vil få tilgang via {{testflight_name}} fra {{apple_name}}.

Hvis du bruker {{ios_name}}, ber vi deg sende inn e-postadressen du bruker for {{app_store_name}} {{form_link_no}}. Under betaen vil appen bare fungere med kontoen din.

{{premium_name}}-medlemmer vil også kunne bli med i mobilfellesskapet vårt. Under betaen blir dette samlingspunktet vårt for feil, funksjonsønsker, UX-endringer og alt imellom. Hvis du vil bli lagt til, gi oss beskjed i skjemaet ovenfor.

Som en påminnelse er dette betaprogramvare. Mange funksjoner vil ikke være på plass ennå, og du bør forvente feil. Tilbakemeldingene dine og hjelpen din med å finne feil vil hjelpe oss med å forbedre oss raskt.

Takk igjen,
{{product_name}}-teamet`,
	[Locales.PL]: `Do wszystkich subskrybentów {{premium_name}}!

Dziękujemy, że pomagacie urzeczywistnić {{product_name}}. Z przyjemnością ogłaszamy uruchomienie wersji beta naszej aplikacji mobilnej. Beta jest dostępna dla wszystkich użytkowników systemu {{android_name}} oraz członków {{premium_name}} korzystających z {{ios_name}}.

Nasza aplikacja mobilna będzie dostępna 15 czerwca 2026 r. W dniu premiery użytkownicy systemu {{android_name}} będą mogli pobrać plik APK z {{github_link}}, a użytkownicy {{ios_name}} uzyskają dostęp przez {{testflight_name}} firmy {{apple_name}}.

Jeśli korzystasz z {{ios_name}}, prześlij adres e-mail używany w {{app_store_name}} {{form_link_pl}}. W trakcie bety aplikacja będzie działać wyłącznie z Twoim kontem.

Członkowie {{premium_name}} będą mogli również dołączyć do naszej społeczności mobilnej. W trakcie bety będzie to nasze centrum zgłaszania błędów, próśb o nowe funkcje, zmian w doświadczeniu użytkownika i wszystkiego pomiędzy. Jeśli chcesz zostać dodany, daj nam znać w powyższym formularzu.

Przypominamy, że jest to oprogramowanie w wersji beta. Wiele funkcji nie będzie jeszcze dostępnych i należy spodziewać się błędów. Wasze opinie i pomoc w znajdowaniu błędów pomogą nam szybko ulepszać aplikację.

Jeszcze raz dziękujemy,
Zespół {{product_name}}`,
	[Locales.PT_BR]: `A todos os assinantes {{premium_name}}!

Obrigado por ajudar a tornar o {{product_name}} realidade. Temos o prazer de anunciar o lançamento da versão beta do nosso app móvel. A versão beta está aberta a todos os usuários de {{android_name}} e aos membros {{premium_name}} no {{ios_name}}.

Nosso app móvel estará disponível em 15 de junho de 2026. No lançamento, os usuários de {{android_name}} poderão baixar o APK pelo {{github_link}}, e os usuários de {{ios_name}} poderão acessá-lo pelo {{testflight_name}} da {{apple_name}}.

Se você usa {{ios_name}}, envie o endereço de e-mail que você usa na {{app_store_name}} {{form_link_pt_br}}. Durante a versão beta, o app funcionará apenas com a sua conta.

Os membros {{premium_name}} também poderão entrar na nossa comunidade móvel. Durante a versão beta, ela será o nosso espaço central para bugs, solicitações de recursos, mudanças de UX e tudo mais. Se quiser ser adicionado, avise-nos no formulário acima.

Como lembrete, este é um software beta. Muitos recursos ainda não estarão disponíveis, e você deve esperar encontrar bugs. Seus comentários e sua ajuda para encontrar bugs nos ajudarão a melhorar rapidamente.

Obrigado mais uma vez,
Equipe {{product_name}}`,
	[Locales.RO]: `Către toți abonații {{premium_name}}!

Vă mulțumim că ne ajutați să transformăm {{product_name}} în realitate. Suntem încântați să anunțăm lansarea versiunii beta a aplicației noastre mobile. Beta este deschisă tuturor utilizatorilor {{android_name}} și membrilor {{premium_name}} pe {{ios_name}}.

Aplicația noastră mobilă va fi disponibilă pe 15 iunie 2026. La lansare, utilizatorii {{android_name}} vor putea descărca APK-ul de pe {{github_link}}, iar utilizatorii {{ios_name}} o vor putea accesa prin {{testflight_name}} de la {{apple_name}}.

Dacă folosiți {{ios_name}}, vă rugăm să trimiteți adresa de e-mail pe care o utilizați pentru {{app_store_name}} {{form_link_ro}}. În timpul perioadei beta, aplicația va funcționa doar cu contul dumneavoastră.

Membrii {{premium_name}} vor putea, de asemenea, să se alăture comunității noastre mobile. În timpul perioadei beta, aceasta va fi centrul nostru pentru buguri, solicitări de funcționalități, modificări de UX și orice altceva. Dacă doriți să fiți adăugat, vă rugăm să ne anunțați în formularul de mai sus.

Ca reamintire, acesta este software beta. Multe funcționalități nu vor fi încă disponibile și ar trebui să vă așteptați la buguri. Feedbackul dumneavoastră și ajutorul în găsirea bugurilor ne vor ajuta să îmbunătățim rapid aplicația.

Vă mulțumim încă o dată,
Echipa {{product_name}}`,
	[Locales.RU]: `Обращаемся ко всем подписчикам {{premium_name}}!

Спасибо, что помогаете воплотить {{product_name}} в жизнь. Мы рады объявить о запуске бета-версии нашего мобильного приложения. Бета доступна всем пользователям {{android_name}} и участникам {{premium_name}} на {{ios_name}}.

Наше мобильное приложение станет доступно 15 июня 2026 года. После запуска пользователи {{android_name}} смогут скачать APK с {{github_link}}, а пользователи {{ios_name}} смогут получить доступ через {{testflight_name}} от {{apple_name}}.

Если вы пользуетесь {{ios_name}}, пожалуйста, отправьте адрес электронной почты, который вы используете для {{app_store_name}}, {{form_link_ru}}. Во время бета-тестирования приложение будет работать только с вашей учетной записью.

Участники {{premium_name}} также смогут присоединиться к нашему мобильному сообществу. Во время бета-тестирования оно станет нашим центром для обсуждения ошибок, запросов на новые функции, изменений UX и всего остального. Если вы хотите, чтобы вас добавили, сообщите нам об этом в форме выше.

Напоминаем, что это бета-версия программного обеспечения. Многие функции пока будут отсутствовать, и следует ожидать ошибок. Ваши отзывы и помощь в поиске ошибок помогут нам быстро улучшать приложение.

Еще раз спасибо,
Команда {{product_name}}`,
	[Locales.SV_SE]: `Till alla {{premium_name}}-prenumeranter!

Tack för att ni hjälper till att göra {{product_name}} möjligt. Vi är glada att kunna meddela lanseringen av betaversionen av vår mobilapp. Betan är öppen för alla {{android_name}}-användare och för {{premium_name}}-medlemmar på {{ios_name}}.

Vår mobilapp blir tillgänglig den 15 juni 2026. Vid lanseringen kommer {{android_name}}-användare att kunna ladda ned APK-filen från {{github_link}}, och {{ios_name}}-användare kommer att kunna få tillgång till appen via {{testflight_name}} från {{apple_name}}.

Om du använder {{ios_name}} ber vi dig skicka in e-postadressen du använder för {{app_store_name}} {{form_link_sv}}. Under betan fungerar appen endast med ditt konto.

{{premium_name}}-medlemmar kommer också att kunna gå med i vår mobilcommunity. Under betan blir detta vår samlingsplats för buggar, funktionsönskemål, UX-ändringar och allt däremellan. Om du vill bli tillagd, meddela oss i formuläret ovan.

Som en påminnelse är detta betaprogramvara. Många funktioner kommer ännu inte att finnas på plats, och du bör räkna med buggar. Din feedback och hjälp med att hitta buggar kommer att hjälpa oss att förbättra appen snabbt.

Tack än en gång,
{{product_name}}-teamet`,
	[Locales.TH]: `ขอเชิญสมาชิก {{premium_name}} ทุกคน!

ขอบคุณที่ช่วยให้ {{product_name}} เกิดขึ้นได้จริง เรารู้สึกยินดีที่จะประกาศการเปิดตัวเบตาของแอปมือถือของเรา เบตานี้เปิดให้ผู้ใช้ {{android_name}} ทุกคนและสมาชิก {{premium_name}} บน {{ios_name}} เข้าร่วมได้

แอปมือถือของเราจะพร้อมให้ใช้งานในวันที่ 15 มิถุนายน 2026 เมื่อเปิดตัว ผู้ใช้ {{android_name}} จะสามารถดาวน์โหลด APK ได้จาก {{github_link}} และผู้ใช้ {{ios_name}} จะสามารถเข้าถึงได้ผ่าน {{testflight_name}} ของ {{apple_name}}

หากคุณเป็นผู้ใช้ {{ios_name}} โปรดส่งที่อยู่อีเมลที่คุณใช้กับ {{app_store_name}} {{form_link_th}} ในช่วงเบตา แอปจะใช้งานได้เฉพาะกับบัญชีของคุณเท่านั้น

สมาชิก {{premium_name}} จะสามารถเข้าร่วมชุมชนมือถือของเราได้ด้วย ในช่วงเบตา ชุมชนนี้จะเป็นศูนย์กลางสำหรับบั๊ก คำขอฟีเจอร์ การเปลี่ยนแปลงด้าน UX และเรื่องอื่น ๆ ทั้งหมด หากคุณต้องการให้เพิ่มคุณเข้าไป โปรดแจ้งให้เราทราบในแบบฟอร์มด้านบน

โปรดทราบว่านี่คือซอฟต์แวร์เบตา หลายฟีเจอร์จะยังไม่พร้อมใช้งาน และคุณควรคาดว่าจะพบข้อผิดพลาด ความคิดเห็นของคุณและความช่วยเหลือในการค้นหาบั๊กจะช่วยให้เราปรับปรุงได้อย่างรวดเร็ว

ขอบคุณอีกครั้ง
ทีม {{product_name}}`,
	[Locales.TR]: `Tüm {{premium_name}} abonelerine çağrımızdır!

{{product_name}}’ın hayata geçmesine yardımcı olduğunuz için teşekkür ederiz. Mobil uygulama betamızı başlatacağımızı duyurmaktan heyecan duyuyoruz. Beta, tüm {{android_name}} kullanıcılarına ve {{ios_name}}’taki {{premium_name}} üyelerine açıktır.

Mobil uygulamamız 15 Haziran 2026’da kullanıma sunulacaktır. Lansmanla birlikte {{android_name}} kullanıcıları APK’yı {{github_link}} üzerinden indirebilecek, {{ios_name}} kullanıcıları ise {{apple_name}}’ın {{testflight_name}} uygulaması üzerinden erişim sağlayabilecektir.

{{ios_name}} kullanıcısıysanız, lütfen {{app_store_name}}’da kullandığınız e-posta adresinizi {{form_link_tr}} gönderin. Beta süresince uygulama yalnızca hesabınızla çalışacaktır.

{{premium_name}} üyeleri ayrıca mobil topluluğumuza katılabilecektir. Beta süresince burası; hatalar, özellik istekleri, UX değişiklikleri ve bunlarla ilgili diğer her şey için merkezimiz olacaktır. Eklenmek isterseniz, lütfen yukarıdaki formda bize bildirin.

Hatırlatmak isteriz ki bu bir beta yazılımdır. Pek çok özellik henüz mevcut olmayacak ve hatalarla karşılaşmayı beklemelisiniz. Geri bildirimleriniz ve hataları bulmamıza yardımcı olmanız, hızlıca gelişmemize katkı sağlayacaktır.

Tekrar teşekkürler,
{{product_name}} Ekibi`,
	[Locales.UK]: `Звертаємося до всіх підписників {{premium_name}}!

Дякуємо, що допомагаєте втілити {{product_name}} у життя. Ми раді оголосити про запуск бета-версії нашого мобільного застосунку. Бета відкрита для всіх користувачів {{android_name}} і для учасників {{premium_name}} на {{ios_name}}.

Наш мобільний застосунок стане доступним 15 червня 2026 року. Після запуску користувачі {{android_name}} зможуть завантажити APK з {{github_link}}, а користувачі {{ios_name}} зможуть отримати доступ через {{testflight_name}} від {{apple_name}}.

Якщо ви користуєтеся {{ios_name}}, будь ласка, надішліть адресу електронної пошти, яку ви використовуєте для {{app_store_name}}, {{form_link_uk}}. Під час бети застосунок працюватиме лише з вашим обліковим записом.

Учасники {{premium_name}} також зможуть приєднатися до нашої мобільної спільноти. Під час бети вона буде нашим осередком для багів, запитів на функції, змін UX і всього іншого. Якщо хочете, щоб вас додали, повідомте нам у формі вище.

Нагадуємо, що це бета-програмне забезпечення. Багатьох функцій ще не буде, і слід очікувати помилок. Ваші відгуки та допомога у виявленні багів допоможуть нам швидко покращувати застосунок.

Ще раз дякуємо,
Команда {{product_name}}`,
	[Locales.VI]: `Kêu gọi tất cả người đăng ký {{premium_name}}!

Cảm ơn bạn đã giúp {{product_name}} trở thành hiện thực. Chúng tôi rất vui được thông báo ra mắt bản beta của ứng dụng di động. Bản beta mở cho tất cả người dùng {{android_name}} và các thành viên {{premium_name}} trên {{ios_name}}.

Ứng dụng di động của chúng tôi sẽ có mặt vào ngày 15 tháng 6 năm 2026. Khi ra mắt, người dùng {{android_name}} sẽ có thể tải APK từ {{github_link}}, còn người dùng {{ios_name}} sẽ có thể truy cập thông qua {{testflight_name}} của {{apple_name}}.

Nếu bạn là người dùng {{ios_name}}, vui lòng gửi địa chỉ email {{app_store_name}} của bạn {{form_link_vi}}. Trong thời gian beta, ứng dụng sẽ chỉ hoạt động với tài khoản của bạn.

Các thành viên {{premium_name}} cũng sẽ có thể tham gia cộng đồng di động của chúng tôi. Trong thời gian beta, đây sẽ là trung tâm của chúng tôi dành cho lỗi, yêu cầu tính năng, thay đổi UX và mọi vấn đề liên quan. Nếu bạn muốn được thêm vào, vui lòng cho chúng tôi biết trong biểu mẫu ở trên.

Xin nhắc lại, đây là phần mềm beta. Nhiều tính năng sẽ chưa có mặt và bạn nên chuẩn bị tinh thần sẽ gặp lỗi. Phản hồi của bạn và sự giúp đỡ của bạn trong việc tìm lỗi sẽ giúp chúng tôi cải thiện nhanh chóng.

Một lần nữa xin cảm ơn,
Đội ngũ {{product_name}}`,
	[Locales.ZH_CN]: `致所有 {{premium_name}} 订阅者！

感谢你们帮助 {{product_name}} 变为现实。我们很高兴地宣布，我们的移动应用 Beta 版即将发布。Beta 版面向所有 {{android_name}} 用户以及 {{ios_name}} 上的 {{premium_name}} 会员开放。

我们的移动应用将于 2026 年 6 月 15 日开放使用。发布后，{{android_name}} 用户可以从 {{github_link}} 下载 APK，{{ios_name}} 用户则可以通过 {{apple_name}} 的 {{testflight_name}} 访问。

如果你是 {{ios_name}} 用户，请在{{form_link_zh_cn}}提交你用于 {{app_store_name}} 的电子邮件地址。在 Beta 期间，该应用只会与你的账号配合使用。

{{premium_name}} 会员还可以加入我们的移动社区。在 Beta 期间，这里将作为我们处理错误反馈、功能请求、用户体验改进以及其他相关事项的中心。如果你希望加入，请在上方表单中告知我们。

请注意，这是 Beta 版软件。许多功能尚未提供，也可能会遇到错误。你们的反馈以及帮助我们发现错误，将帮助我们快速改进。

再次感谢，
{{product_name}} 团队`,
	[Locales.ZH_TW]: `致所有 {{premium_name}} 訂閱者！

感謝你們協助讓 {{product_name}} 成真。我們很高興宣布，我們的行動應用程式 Beta 版即將推出。Beta 版開放給所有 {{android_name}} 使用者，以及 {{ios_name}} 上的 {{premium_name}} 會員。

我們的行動應用程式將於 2026 年 6 月 15 日開放使用。推出時，{{android_name}} 使用者可以從 {{github_link}} 下載 APK，{{ios_name}} 使用者則可透過 {{apple_name}} 的 {{testflight_name}} 存取。

如果你是 {{ios_name}} 使用者，請在{{form_link_zh_tw}}提交你用於 {{app_store_name}} 的電子郵件地址。在 Beta 期間，應用程式只會與你的帳號搭配使用。

{{premium_name}} 會員也可以加入我們的行動社群。在 Beta 期間，這裡將作為我們處理錯誤回報、功能請求、UX 變更以及其他相關事項的中心。如果你希望加入，請在上方表單中告知我們。

提醒你，這是 Beta 版軟體。許多功能尚未提供，也可能會遇到錯誤。你的意見回饋以及協助我們找出錯誤，將幫助我們快速改進。

再次感謝，
{{product_name}} 團隊`,
};

const MOBILE_COMMUNITY_FORM_CLARIFICATION_TEMPLATES: Partial<Record<LocaleCode, string>> = {
	[Locales.AR]:
		'النموذج مخصص أيضًا للوصول إلى مجتمع الهاتف المحمول على {{android_name}} و{{ios_name}}. لإضافتك إلى مجتمع الهاتف المحمول، نحتاج فقط إلى معرّف مستخدم {{product_name}} الخاص بك في النموذج: {{user_id}}.',
	[Locales.BG]:
		'Формулярът е и за достъп до мобилната общност на {{android_name}} и {{ios_name}}. За да бъдете добавени към мобилната общност, във формуляра ни е нужен само вашият потребителски ID в {{product_name}}: {{user_id}}.',
	[Locales.CS]:
		'Formulář slouží také pro přístup do mobilní komunity na {{android_name}} a {{ios_name}}. Pro přidání do mobilní komunity potřebujeme ve formuláři pouze vaše uživatelské ID v {{product_name}}: {{user_id}}.',
	[Locales.DA]:
		'Formularen bruges også til adgang til mobilfællesskabet på {{android_name}} og {{ios_name}}. For at blive tilføjet til mobilfællesskabet behøver vi kun dit {{product_name}}-bruger-id i formularen: {{user_id}}.',
	[Locales.DE]:
		'Das Formular ist auch für den Zugang zur mobilen Community auf {{android_name}} und {{ios_name}} gedacht. Um zur mobilen Community hinzugefügt zu werden, benötigen wir im Formular nur deine {{product_name}}-Benutzer-ID: {{user_id}}.',
	[Locales.EL]:
		'Η φόρμα είναι επίσης για πρόσβαση στην κοινότητα κινητών σε {{android_name}} και {{ios_name}}. Για να προστεθείτε στην κοινότητα κινητών, χρειαζόμαστε στη φόρμα μόνο το αναγνωριστικό χρήστη σας στο {{product_name}}: {{user_id}}.',
	[Locales.EN_GB]:
		'The form is also for mobile community access on {{android_name}} and {{ios_name}}. To be added to the mobile community, we only need your {{product_name}} user ID in the form: {{user_id}}.',
	[Locales.EN_US]:
		'The form is also for mobile community access on {{android_name}} and {{ios_name}}. To be added to the mobile community, we only need your {{product_name}} user ID in the form: {{user_id}}.',
	[Locales.ES_ES]:
		'El formulario también sirve para acceder a la comunidad móvil en {{android_name}} y {{ios_name}}. Para añadirte a la comunidad móvil, solo necesitamos tu ID de usuario de {{product_name}} en el formulario: {{user_id}}.',
	[Locales.ES_419]:
		'El formulario también sirve para acceder a la comunidad móvil en {{android_name}} y {{ios_name}}. Para agregarte a la comunidad móvil, solo necesitamos tu ID de usuario de {{product_name}} en el formulario: {{user_id}}.',
	[Locales.FI]:
		'Lomake on tarkoitettu myös mobiiliyhteisön käyttöoikeuteen {{android_name}}- ja {{ios_name}}-käyttäjille. Jotta voimme lisätä sinut mobiiliyhteisöön, tarvitsemme lomakkeessa vain {{product_name}}-käyttäjätunnuksesi: {{user_id}}.',
	[Locales.FR]:
		'Le formulaire sert aussi à demander l’accès à la communauté mobile sur {{android_name}} et {{ios_name}}. Pour être ajouté à la communauté mobile, nous avons seulement besoin de votre identifiant utilisateur {{product_name}} dans le formulaire : {{user_id}}.',
	[Locales.HE]:
		'הטופס מיועד גם לקבלת גישה לקהילת המובייל ב-{{android_name}} וב-{{ios_name}}. כדי להוסיף אתכם לקהילת המובייל, אנחנו צריכים בטופס רק את מזהה המשתמש שלכם ב-{{product_name}}: {{user_id}}.',
	[Locales.HI]:
		'फ़ॉर्म {{android_name}} और {{ios_name}} पर मोबाइल समुदाय की पहुँच के लिए भी है। मोबाइल समुदाय में जोड़े जाने के लिए, हमें फ़ॉर्म में केवल आपका {{product_name}} उपयोगकर्ता ID चाहिए: {{user_id}}.',
	[Locales.HR]:
		'Obrazac služi i za pristup mobilnoj zajednici na {{android_name}} i {{ios_name}}. Za dodavanje u mobilnu zajednicu u obrascu nam treba samo vaš korisnički ID za {{product_name}}: {{user_id}}.',
	[Locales.HU]:
		'Az űrlap a mobilos közösséghez való hozzáférésre is szolgál {{android_name}} és {{ios_name}} esetén. Ahhoz, hogy hozzáadjunk a mobilos közösséghez, az űrlapon csak a {{product_name}} felhasználói azonosítódra van szükségünk: {{user_id}}.',
	[Locales.ID]:
		'Formulir ini juga digunakan untuk akses komunitas seluler di {{android_name}} dan {{ios_name}}. Agar Anda dapat ditambahkan ke komunitas seluler, kami hanya membutuhkan ID pengguna {{product_name}} Anda di formulir: {{user_id}}.',
	[Locales.IT]:
		'Il modulo serve anche per l’accesso alla community mobile su {{android_name}} e {{ios_name}}. Per aggiungerti alla community mobile, nel modulo ci serve solo il tuo ID utente {{product_name}}: {{user_id}}.',
	[Locales.JA]:
		'このフォームは、{{android_name}}と{{ios_name}}のモバイルコミュニティへのアクセス申請にも使用します。モバイルコミュニティに追加するためにフォームで必要なのは、あなたの{{product_name}}ユーザーIDだけです: {{user_id}}。',
	[Locales.KO]:
		'이 양식은 {{android_name}} 및 {{ios_name}} 모바일 커뮤니티 접근 신청에도 사용됩니다. 모바일 커뮤니티에 추가하려면 양식에 {{product_name}} 사용자 ID만 입력하면 됩니다: {{user_id}}.',
	[Locales.LT]:
		'Forma taip pat skirta prieigai prie mobiliosios bendruomenės naudojant {{android_name}} ir {{ios_name}}. Kad galėtume jus pridėti prie mobiliosios bendruomenės, formoje mums reikia tik jūsų {{product_name}} naudotojo ID: {{user_id}}.',
	[Locales.NL]:
		'Het formulier is ook bedoeld voor toegang tot de mobiele community op {{android_name}} en {{ios_name}}. Om je aan de mobiele community toe te voegen, hebben we in het formulier alleen je {{product_name}}-gebruikers-ID nodig: {{user_id}}.',
	[Locales.NO]:
		'Skjemaet brukes også for tilgang til mobilfellesskapet på {{android_name}} og {{ios_name}}. For å bli lagt til i mobilfellesskapet trenger vi bare {{product_name}}-bruker-ID-en din i skjemaet: {{user_id}}.',
	[Locales.PL]:
		'Formularz służy także do uzyskania dostępu do społeczności mobilnej na {{android_name}} i {{ios_name}}. Aby dodać Cię do społeczności mobilnej, w formularzu potrzebujemy tylko Twojego identyfikatora użytkownika {{product_name}}: {{user_id}}.',
	[Locales.PT_BR]:
		'O formulário também serve para acesso à comunidade móvel no {{android_name}} e no {{ios_name}}. Para adicionar você à comunidade móvel, precisamos apenas do seu ID de usuário do {{product_name}} no formulário: {{user_id}}.',
	[Locales.RO]:
		'Formularul este și pentru accesul la comunitatea mobilă pe {{android_name}} și {{ios_name}}. Pentru a vă adăuga în comunitatea mobilă, avem nevoie în formular doar de ID-ul dvs. de utilizator {{product_name}}: {{user_id}}.',
	[Locales.RU]:
		'Форма также предназначена для доступа к мобильному сообществу на {{android_name}} и {{ios_name}}. Чтобы добавить вас в мобильное сообщество, в форме нам нужен только ваш пользовательский ID в {{product_name}}: {{user_id}}.',
	[Locales.SV_SE]:
		'Formuläret används också för åtkomst till mobilcommunityn på {{android_name}} och {{ios_name}}. För att läggas till i mobilcommunityn behöver vi bara ditt {{product_name}}-användar-ID i formuläret: {{user_id}}.',
	[Locales.TH]:
		'แบบฟอร์มนี้ใช้สำหรับการเข้าถึงชุมชนมือถือบน {{android_name}} และ {{ios_name}} ด้วย หากต้องการให้เราเพิ่มคุณในชุมชนมือถือ เราต้องการเพียง ID ผู้ใช้ {{product_name}} ของคุณในแบบฟอร์ม: {{user_id}}',
	[Locales.TR]:
		'Form, {{android_name}} ve {{ios_name}} üzerindeki mobil topluluk erişimi için de kullanılır. Mobil topluluğa eklenmeniz için formda yalnızca {{product_name}} kullanıcı ID’nize ihtiyacımız var: {{user_id}}.',
	[Locales.UK]:
		'Форма також призначена для доступу до мобільної спільноти на {{android_name}} та {{ios_name}}. Щоб додати вас до мобільної спільноти, у формі нам потрібен лише ваш ID користувача {{product_name}}: {{user_id}}.',
	[Locales.VI]:
		'Biểu mẫu cũng dùng để yêu cầu quyền truy cập cộng đồng di động trên {{android_name}} và {{ios_name}}. Để thêm bạn vào cộng đồng di động, chúng tôi chỉ cần ID người dùng {{product_name}} của bạn trong biểu mẫu: {{user_id}}.',
	[Locales.ZH_CN]:
		'该表单也用于申请 {{android_name}} 和 {{ios_name}} 移动社区访问权限。若要加入移动社区，我们在表单中只需要你的 {{product_name}} 用户 ID：{{user_id}}。',
	[Locales.ZH_TW]:
		'此表單也用於申請 {{android_name}} 和 {{ios_name}} 行動社群存取權。若要加入行動社群，我們在表單中只需要你的 {{product_name}} 使用者 ID：{{user_id}}。',
};

const CORRECTION_HEADER_TEMPLATES: Partial<Record<LocaleCode, string>> = {
	[Locales.AR]: 'تصحيح سريع لرسالتنا السابقة عن النسخة التجريبية لتطبيق الهاتف المحمول:',
	[Locales.BG]: 'Кратка корекция към предишното ни съобщение за мобилната бета:',
	[Locales.CS]: 'Krátké upřesnění k naší předchozí zprávě o mobilní betě:',
	[Locales.DA]: 'En hurtig rettelse til vores tidligere besked om mobilbetaen:',
	[Locales.DE]: 'Kurze Korrektur zu unserer vorherigen Nachricht zur mobilen Beta:',
	[Locales.EL]: 'Μια γρήγορη διόρθωση στο προηγούμενο μήνυμά μας για τη beta των κινητών:',
	[Locales.EN_GB]: 'Quick correction to our previous mobile beta message:',
	[Locales.EN_US]: 'Quick correction to our previous mobile beta message:',
	[Locales.ES_ES]: 'Una corrección rápida a nuestro mensaje anterior sobre la beta móvil:',
	[Locales.ES_419]: 'Una corrección rápida a nuestro mensaje anterior sobre la beta móvil:',
	[Locales.FI]: 'Nopea korjaus aiempaan mobiilibetaa koskevaan viestiimme:',
	[Locales.FR]: 'Petite correction à notre précédent message sur la bêta mobile :',
	[Locales.HE]: 'תיקון קצר להודעה הקודמת שלנו על גרסת הבטא למובייל:',
	[Locales.HI]: 'हमारे पिछले मोबाइल बीटा संदेश में एक छोटा सुधार:',
	[Locales.HR]: 'Kratka ispravka naše prethodne poruke o mobilnoj beti:',
	[Locales.HU]: 'Rövid pontosítás a korábbi mobilbétás üzenetünkhöz:',
	[Locales.ID]: 'Koreksi singkat untuk pesan beta seluler kami sebelumnya:',
	[Locales.IT]: 'Una rapida correzione al nostro precedente messaggio sulla beta mobile:',
	[Locales.JA]: '以前のモバイルベータに関するメッセージについて、簡単な訂正です:',
	[Locales.KO]: '이전 모바일 베타 메시지에 대한 간단한 정정입니다:',
	[Locales.LT]: 'Trumpas ankstesnio pranešimo apie mobiliąją beta versiją patikslinimas:',
	[Locales.NL]: 'Een korte correctie op ons eerdere bericht over de mobiele bèta:',
	[Locales.NO]: 'En rask rettelse til den forrige meldingen vår om mobilbetaen:',
	[Locales.PL]: 'Krótka korekta do naszej poprzedniej wiadomości o becie mobilnej:',
	[Locales.PT_BR]: 'Uma correção rápida sobre a nossa mensagem anterior da beta móvel:',
	[Locales.RO]: 'O corectare rapidă la mesajul nostru anterior despre beta mobilă:',
	[Locales.RU]: 'Небольшое уточнение к нашему предыдущему сообщению о мобильной бете:',
	[Locales.SV_SE]: 'En snabb rättelse till vårt tidigare meddelande om mobilbetan:',
	[Locales.TH]: 'ขอแก้ไขสั้น ๆ เกี่ยวกับข้อความเบตาแอปมือถือก่อนหน้านี้:',
	[Locales.TR]: 'Önceki mobil beta mesajımızla ilgili kısa bir düzeltme:',
	[Locales.UK]: 'Коротке уточнення до нашого попереднього повідомлення про мобільну бету:',
	[Locales.VI]: 'Một đính chính nhanh cho thông báo beta di động trước đó của chúng tôi:',
	[Locales.ZH_CN]: '对我们之前移动 Beta 消息的简短更正：',
	[Locales.ZH_TW]: '對我們先前行動 Beta 訊息的簡短更正：',
};

const FORM_LINK_SENTENCE_TEMPLATES: Partial<Record<LocaleCode, string>> = {
	[Locales.AR]: 'يمكنك استخدام النموذج {{form_link}}.',
	[Locales.BG]: 'Можете да използвате формуляра {{form_link}}.',
	[Locales.CS]: 'Formulář můžete použít {{form_link}}.',
	[Locales.DA]: 'Du kan bruge formularen {{form_link}}.',
	[Locales.DE]: 'Du kannst das Formular {{form_link}} verwenden.',
	[Locales.EL]: 'Μπορείτε να χρησιμοποιήσετε τη φόρμα {{form_link}}.',
	[Locales.EN_GB]: 'You can use the form {{form_link}}.',
	[Locales.EN_US]: 'You can use the form {{form_link}}.',
	[Locales.ES_ES]: 'Puedes usar el formulario {{form_link}}.',
	[Locales.ES_419]: 'Puedes usar el formulario {{form_link}}.',
	[Locales.FI]: 'Voit käyttää lomaketta {{form_link}}.',
	[Locales.FR]: 'Vous pouvez utiliser le formulaire {{form_link}}.',
	[Locales.HE]: 'אפשר להשתמש בטופס {{form_link}}.',
	[Locales.HI]: 'आप फ़ॉर्म {{form_link}} इस्तेमाल कर सकते हैं.',
	[Locales.HR]: 'Možete upotrijebiti obrazac {{form_link}}.',
	[Locales.HU]: 'Az űrlapot {{form_link}} használhatod.',
	[Locales.ID]: 'Anda dapat menggunakan formulir {{form_link}}.',
	[Locales.IT]: 'Puoi usare il modulo {{form_link}}.',
	[Locales.JA]: 'フォームは{{form_link}}から使用できます。',
	[Locales.KO]: '양식은 {{form_link}}에서 사용할 수 있습니다.',
	[Locales.LT]: 'Formą galite naudoti {{form_link}}.',
	[Locales.NL]: 'Je kunt het formulier {{form_link}} gebruiken.',
	[Locales.NO]: 'Du kan bruke skjemaet {{form_link}}.',
	[Locales.PL]: 'Możesz użyć formularza {{form_link}}.',
	[Locales.PT_BR]: 'Você pode usar o formulário {{form_link}}.',
	[Locales.RO]: 'Puteți folosi formularul {{form_link}}.',
	[Locales.RU]: 'Вы можете использовать форму {{form_link}}.',
	[Locales.SV_SE]: 'Du kan använda formuläret {{form_link}}.',
	[Locales.TH]: 'คุณสามารถใช้แบบฟอร์ม {{form_link}}',
	[Locales.TR]: 'Formu {{form_link}} kullanabilirsiniz.',
	[Locales.UK]: 'Ви можете скористатися формою {{form_link}}.',
	[Locales.VI]: 'Bạn có thể dùng biểu mẫu {{form_link}}.',
	[Locales.ZH_CN]: '你可以使用表单{{form_link}}。',
	[Locales.ZH_TW]: '你可以使用表單{{form_link}}。',
};

const IOS_APP_ACCESS_REMINDER_TEMPLATES: Partial<Record<LocaleCode, string>> = {
	[Locales.AR]:
		'إذا كنت تحتاج أيضًا إلى الوصول إلى النسخة التجريبية من تطبيق {{ios_name}}، فيُرجى الاستمرار في تضمين عنوان بريدك الإلكتروني المستخدم في {{app_store_name}} داخل النموذج.',
	[Locales.BG]:
		'Ако ви е нужен и достъп до бета приложението за {{ios_name}}, все пак включете във формуляра имейл адреса си за {{app_store_name}}.',
	[Locales.CS]:
		'Pokud potřebujete také přístup k beta aplikaci pro {{ios_name}}, stále do formuláře uveďte svou e-mailovou adresu pro {{app_store_name}}.',
	[Locales.DA]:
		'Hvis du også har brug for adgang til {{ios_name}}-appbetaen, skal du stadig medtage din {{app_store_name}}-e-mailadresse i formularen.',
	[Locales.DE]:
		'Wenn du auch Zugriff auf die {{ios_name}}-App-Beta brauchst, gib im Formular weiterhin deine E-Mail-Adresse für den {{app_store_name}} an.',
	[Locales.EL]:
		'Αν χρειάζεστε επίσης πρόσβαση στη beta εφαρμογή για {{ios_name}}, συνεχίστε να συμπεριλαμβάνετε στη φόρμα τη διεύθυνση email που χρησιμοποιείτε για το {{app_store_name}}.',
	[Locales.EN_GB]:
		'If you also need {{ios_name}} app beta access, still include your {{app_store_name}} email address in the form.',
	[Locales.EN_US]:
		'If you also need {{ios_name}} app beta access, still include your {{app_store_name}} email address in the form.',
	[Locales.ES_ES]:
		'Si también necesitas acceso a la beta de la aplicación para {{ios_name}}, sigue incluyendo en el formulario tu dirección de correo del {{app_store_name}}.',
	[Locales.ES_419]:
		'Si también necesitas acceso a la beta de la app para {{ios_name}}, sigue incluyendo en el formulario tu dirección de correo de {{app_store_name}}.',
	[Locales.FI]:
		'Jos tarvitset myös pääsyn {{ios_name}}-sovelluksen betaan, lisää lomakkeeseen edelleen {{app_store_name}} -sähköpostiosoitteesi.',
	[Locales.FR]:
		'Si vous avez aussi besoin de l’accès à la bêta de l’application {{ios_name}}, indiquez toujours dans le formulaire l’adresse e-mail utilisée pour l’{{app_store_name}}.',
	[Locales.HE]:
		'אם אתם צריכים גם גישה לבטא של אפליקציית {{ios_name}}, עדיין יש לכלול בטופס את כתובת הדוא״ל שלכם עבור {{app_store_name}}.',
	[Locales.HI]:
		'यदि आपको {{ios_name}} ऐप बीटा एक्सेस भी चाहिए, तो फ़ॉर्म में अपना {{app_store_name}} ईमेल पता अभी भी शामिल करें.',
	[Locales.HR]:
		'Ako vam treba i pristup beta verziji aplikacije za {{ios_name}}, u obrascu i dalje navedite svoju adresu e-pošte za {{app_store_name}}.',
	[Locales.HU]:
		'Ha az {{ios_name}}-app bétájához is hozzáférést szeretnél, továbbra is add meg az űrlapon a {{app_store_name}} e-mail-címedet.',
	[Locales.ID]:
		'Jika Anda juga membutuhkan akses beta aplikasi {{ios_name}}, tetap sertakan alamat email {{app_store_name}} Anda di formulir.',
	[Locales.IT]:
		'Se ti serve anche l’accesso alla beta dell’app per {{ios_name}}, includi comunque nel modulo l’indirizzo email che usi per l’{{app_store_name}}.',
	[Locales.JA]:
		'{{ios_name}}アプリのベータアクセスも必要な場合は、{{app_store_name}}で使用しているメールアドレスもフォームに入力してください。',
	[Locales.KO]: '{{ios_name}} 앱 베타 접근도 필요하다면 양식에 {{app_store_name}} 이메일 주소도 계속 포함해 주세요.',
	[Locales.LT]:
		'Jei jums taip pat reikia prieigos prie {{ios_name}} programėlės beta versijos, formoje vis tiek nurodykite savo {{app_store_name}} el. pašto adresą.',
	[Locales.NL]:
		'Als je ook toegang tot de {{ios_name}}-appbèta nodig hebt, vermeld dan nog steeds je e-mailadres voor de {{app_store_name}} in het formulier.',
	[Locales.NO]:
		'Hvis du også trenger tilgang til {{ios_name}}-appbetaen, må du fortsatt ta med e-postadressen din for {{app_store_name}} i skjemaet.',
	[Locales.PL]:
		'Jeśli potrzebujesz także dostępu do bety aplikacji na {{ios_name}}, nadal podaj w formularzu adres e-mail używany w {{app_store_name}}.',
	[Locales.PT_BR]:
		'Se você também precisa de acesso à beta do app para {{ios_name}}, ainda inclua no formulário o endereço de e-mail que você usa na {{app_store_name}}.',
	[Locales.RO]:
		'Dacă aveți nevoie și de acces la beta aplicației pentru {{ios_name}}, includeți în continuare în formular adresa de e-mail folosită pentru {{app_store_name}}.',
	[Locales.RU]:
		'Если вам также нужен доступ к бета-версии приложения для {{ios_name}}, по-прежнему укажите в форме адрес электронной почты для {{app_store_name}}.',
	[Locales.SV_SE]:
		'Om du också behöver åtkomst till {{ios_name}}-appbetan ska du fortfarande ange e-postadressen du använder för {{app_store_name}} i formuläret.',
	[Locales.TH]: 'หากคุณต้องการเข้าถึงเบตาของแอป {{ios_name}} ด้วย โปรดยังระบุอีเมล {{app_store_name}} ของคุณในแบบฟอร์ม',
	[Locales.TR]:
		'{{ios_name}} uygulama betasına da erişim gerekiyorsa, formda {{app_store_name}} e-posta adresinizi yine de ekleyin.',
	[Locales.UK]:
		'Якщо вам також потрібен доступ до бета-версії застосунку для {{ios_name}}, усе одно вкажіть у формі адресу електронної пошти для {{app_store_name}}.',
	[Locales.VI]:
		'Nếu bạn cũng cần quyền truy cập beta ứng dụng {{ios_name}}, hãy vẫn điền địa chỉ email {{app_store_name}} của bạn trong biểu mẫu.',
	[Locales.ZH_CN]:
		'如果你还需要 {{ios_name}} 应用 Beta 访问权限，请仍在表单中填写你的 {{app_store_name}} 电子邮件地址。',
	[Locales.ZH_TW]:
		'如果你也需要 {{ios_name}} 應用程式 Beta 存取權，請仍在表單中填寫你的 {{app_store_name}} 電子郵件地址。',
};

const CORRECTION_OUTRO_TEMPLATES: Partial<Record<LocaleCode, string>> = {
	[Locales.AR]: 'نأسف على الالتباس،\nفريق {{product_name}}',
	[Locales.BG]: 'Извиняваме се за объркването,\nЕкипът на {{product_name}}',
	[Locales.CS]: 'Omlouváme se za zmatek,\nTým {{product_name}}',
	[Locales.DA]: 'Beklager forvirringen,\n{{product_name}}-teamet',
	[Locales.DE]: 'Entschuldige die Verwirrung,\nDas {{product_name}}-Team',
	[Locales.EL]: 'Συγγνώμη για τη σύγχυση,\nΗ ομάδα του {{product_name}}',
	[Locales.EN_GB]: 'Sorry for the confusion,\nThe {{product_name}} Team',
	[Locales.EN_US]: 'Sorry for the confusion,\nThe {{product_name}} Team',
	[Locales.ES_ES]: 'Perdón por la confusión,\nEl equipo de {{product_name}}',
	[Locales.ES_419]: 'Perdón por la confusión,\nEl equipo de {{product_name}}',
	[Locales.FI]: 'Pahoittelut epäselvyydestä,\n{{product_name}}-tiimi',
	[Locales.FR]: 'Désolé pour la confusion,\nL’équipe {{product_name}}',
	[Locales.HE]: 'סליחה על הבלבול,\nצוות {{product_name}}',
	[Locales.HI]: 'भ्रम के लिए क्षमा करें,\n{{product_name}} टीम',
	[Locales.HR]: 'Ispričavamo se zbog zabune,\n{{product_name}} tim',
	[Locales.HU]: 'Elnézést a félreértésért,\nA {{product_name}} csapata',
	[Locales.ID]: 'Mohon maaf atas kebingungannya,\nTim {{product_name}}',
	[Locales.IT]: 'Ci scusiamo per la confusione,\nIl team {{product_name}}',
	[Locales.JA]: '混乱させてしまい申し訳ありません。\n{{product_name}}チーム',
	[Locales.KO]: '혼란을 드려 죄송합니다.\n{{product_name}} 팀',
	[Locales.LT]: 'Atsiprašome už neaiškumą,\n{{product_name}} komanda',
	[Locales.NL]: 'Sorry voor de verwarring,\nHet {{product_name}}-team',
	[Locales.NO]: 'Beklager forvirringen,\n{{product_name}}-teamet',
	[Locales.PL]: 'Przepraszamy za zamieszanie,\nZespół {{product_name}}',
	[Locales.PT_BR]: 'Desculpe pela confusão,\nEquipe {{product_name}}',
	[Locales.RO]: 'Ne pare rău pentru confuzie,\nEchipa {{product_name}}',
	[Locales.RU]: 'Извините за путаницу,\nКоманда {{product_name}}',
	[Locales.SV_SE]: 'Ursäkta förvirringen,\n{{product_name}}-teamet',
	[Locales.TH]: 'ขออภัยสำหรับความสับสน\nทีม {{product_name}}',
	[Locales.TR]: 'Karışıklık için özür dileriz,\n{{product_name}} Ekibi',
	[Locales.UK]: 'Вибачте за плутанину,\nКоманда {{product_name}}',
	[Locales.VI]: 'Xin lỗi vì sự nhầm lẫn,\nĐội ngũ {{product_name}}',
	[Locales.ZH_CN]: '很抱歉造成混淆，\n{{product_name}} 团队',
	[Locales.ZH_TW]: '很抱歉造成混淆，\n{{product_name}} 團隊',
};

function getTemplate(
	templates: Partial<Record<LocaleCode, string>>,
	locale: LocaleCode | string | null | undefined,
): string {
	return templates[locale as LocaleCode] ?? templates[Locales.EN_US] ?? '';
}

function renderTemplate(template: string, locale: LocaleCode | string | null | undefined, userId: string): string {
	const tokenValues = buildTokenValues(locale, userId);
	return template.replace(/\{\{([a-z0-9_]+)\}\}/g, (match, key: string) => tokenValues[key] ?? match);
}

function insertBeforeBetaReminder(template: string, paragraph: string): string {
	const paragraphs = template.split('\n\n');
	if (paragraphs.length < 3) {
		return `${template}\n\n${paragraph}`;
	}
	return [...paragraphs.slice(0, -2), paragraph, ...paragraphs.slice(-2)].join('\n\n');
}

export function resolvePlutoniumMobileBetaDispatchBody(
	locale: LocaleCode | string | null | undefined,
	userId: string,
): string {
	const template =
		locale === Locales.EN_GB || locale === Locales.EN_US
			? ENGLISH_TEMPLATE
			: (LOCALIZED_TEMPLATES[locale as LocaleCode] ?? ENGLISH_TEMPLATE);
	const clarification = getTemplate(MOBILE_COMMUNITY_FORM_CLARIFICATION_TEMPLATES, locale);
	return renderTemplate(insertBeforeBetaReminder(template, clarification), locale, userId);
}

export function resolvePlutoniumMobileBetaCorrectionBody(
	locale: LocaleCode | string | null | undefined,
	userId: string,
): string {
	const template = [
		getTemplate(CORRECTION_HEADER_TEMPLATES, locale),
		getTemplate(MOBILE_COMMUNITY_FORM_CLARIFICATION_TEMPLATES, locale),
		getTemplate(FORM_LINK_SENTENCE_TEMPLATES, locale),
		getTemplate(IOS_APP_ACCESS_REMINDER_TEMPLATES, locale),
		getTemplate(CORRECTION_OUTRO_TEMPLATES, locale),
	].join('\n\n');
	return renderTemplate(template, locale, userId);
}
