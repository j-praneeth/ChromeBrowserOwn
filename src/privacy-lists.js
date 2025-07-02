// Privacy and Ad Blocking Lists
// This file contains default privacy protection rules

const PrivacyLists = {
    // Common tracking domains to block
    trackingDomains: [
        'google-analytics.com',
        'googletagmanager.com',
        'facebook.com',
        'doubleclick.net',
        'googlesyndication.com',
        'amazon-adsystem.com',
        'twitter.com',
        'instagram.com',
        'linkedin.com',
        'pinterest.com',
        'snapchat.com',
        'tiktok.com',
        'outbrain.com',
        'taboola.com',
        'criteo.com',
        'adsystem.com',
        'adsymptotic.com',
        'adsservicing.com',
        'scorecardresearch.com',
        'quantserve.com',
        'hotjar.com',
        'mouseflow.com',
        'crazyegg.com',
        'fullstory.com',
        'mixpanel.com',
        'segment.com',
        'amplitude.com',
        'heap.com',
        'kissmetrics.com',
        'chartbeat.com',
        'newrelic.com',
        'bugsnag.com',
        'sentry.io',
        'rollbar.com',
        'optimizely.com',
        'googleoptimize.com',
        'vwo.com',
        'unbounce.com',
        'leadpages.com',
        'mailchimp.com',
        'constantcontact.com',
        'hubspot.com',
        'marketo.com',
        'pardot.com',
        'salesforce.com',
        'zendesk.com',
        'intercom.io',
        'drift.com',
        'tawk.to',
        'livechatinc.com',
        'zopim.com',
        'olark.com'
    ],

    // Common ad serving domains
    adDomains: [
        'googlesyndication.com',
        'googleadservices.com',
        'amazon-adsystem.com',
        'adsystem.com',
        'doubleclick.net',
        'media.net',
        'adsymptotic.com',
        'adsservicing.com',
        'outbrain.com',
        'taboola.com',
        'revcontent.com',
        'criteo.com',
        'bidswitch.net',
        'adswifit.com',
        'casalemedia.com',
        'contextweb.com',
        'rubiconproject.com',
        'openx.net',
        'pubmatic.com',
        'adsense.com',
        'adskeeper.com',
        'adnxs.com',
        'adsupply.com',
        'advertising.com',
        'adsupply.com',
        'adform.net',
        'adnxs.com',
        'adsupply.com',
        'adswizz.com',
        'adsupply.com',
        'adtech.de',
        'adform.net',
        'addthis.com',
        'sharethis.com',
        '2mdn.net',
        'adsupply.com',
        'adsupply.com',
        'adsupply.com'
    ],

    // URL patterns that typically indicate tracking or ads
    blockPatterns: [
        /.*\.ads\./,
        /.*\.advertising\./,
        /.*\.tracker\./,
        /.*\.analytics\./,
        /.*\.metrics\./,
        /.*\.telemetry\./,
        /.*\.beacon\./,
        /.*\.pixel\./,
        /.*googletagmanager.*/,
        /.*google-analytics.*/,
        /.*facebook\.com\/tr.*/,
        /.*\.doubleclick\.net.*/,
        /.*googlesyndication.*/,
        /.*\/ads\/.*/,
        /.*\/advertising\/.*/,
        /.*\/tracker\/.*/,
        /.*\/analytics\/.*/,
        /.*\/ga\/.*/,
        /.*\/gtm\/.*/,
        /.*\/pixel\/.*/,
        /.*\/beacon\/.*/,
        /.*\/telemetry\/.*/,
        /.*\/metrics\/.*/,
        /.*\/track\/.*/,
        /.*\/tracking\/.*/,
        /.*\/advert.*/,
        /.*\/advertisement.*/,
        /.*\/adsystem.*/,
        /.*\/adservice.*/,
        /.*\/adserver.*/,
        /.*\/adserving.*/,
        /.*banners?.*/,
        /.*\/popunder.*/,
        /.*\/popup.*/,
        /.*\/affiliate.*/
    ],

    // Safe domains that should never be blocked
    safeDomains: [
        'localhost',
        '127.0.0.1',
        'github.com',
        'stackoverflow.com',
        'mozilla.org',
        'w3.org',
        'wikipedia.org',
        'duckduckgo.com',
        'startpage.com',
        'searx.org',
        'yandex.com',
        'bing.com',
        'archive.org',
        'wayback.archive.org',
        'creativecommons.org',
        'opensource.org',
        'fsf.org',
        'eff.org',
        'privacytools.io',
        'privacyguides.org',
        'torproject.org',
        'signal.org',
        'protonmail.com',
        'tutanota.com',
        'brave.com',
        'firefox.com',
        'chromium.org'
    ],

    // Check if a URL should be blocked
    shouldBlockUrl(url) {
        try {
            const urlObj = new URL(url);
            const hostname = urlObj.hostname.toLowerCase();
            const fullUrl = url.toLowerCase();

            // Never block safe domains
            if (this.safeDomains.some(domain => hostname.includes(domain))) {
                return false;
            }

            // Check if hostname is in tracking domains
            if (this.trackingDomains.some(domain => hostname.includes(domain))) {
                return true;
            }

            // Check if hostname is in ad domains
            if (this.adDomains.some(domain => hostname.includes(domain))) {
                return true;
            }

            // Check against URL patterns
            if (this.blockPatterns.some(pattern => pattern.test(fullUrl))) {
                return true;
            }

            return false;

        } catch (error) {
            console.error('Error checking URL:', error);
            return false;
        }
    },

    // Get the reason why a URL was blocked
    getBlockReason(url) {
        try {
            const urlObj = new URL(url);
            const hostname = urlObj.hostname.toLowerCase();
            const fullUrl = url.toLowerCase();

            if (this.trackingDomains.some(domain => hostname.includes(domain))) {
                return 'Tracking domain';
            }

            if (this.adDomains.some(domain => hostname.includes(domain))) {
                return 'Advertisement domain';
            }

            const matchedPattern = this.blockPatterns.find(pattern => pattern.test(fullUrl));
            if (matchedPattern) {
                return 'Blocked URL pattern';
            }

            return 'Unknown';

        } catch (error) {
            return 'Invalid URL';
        }
    },

    // Update lists from remote sources (called by Rust backend)
    async updateLists() {
        try {
            // This would typically fetch updated lists from sources like EasyList
            console.log('Privacy lists updated from local defaults');
            return true;
        } catch (error) {
            console.error('Failed to update privacy lists:', error);
            return false;
        }
    },

    // Get statistics about blocked content
    getStats() {
        return {
            trackingDomains: this.trackingDomains.length,
            adDomains: this.adDomains.length,
            blockPatterns: this.blockPatterns.length,
            safeDomains: this.safeDomains.length
        };
    }
};

// Export for use in other parts of the application
if (typeof module !== 'undefined' && module.exports) {
    module.exports = PrivacyLists;
} else {
    window.PrivacyLists = PrivacyLists;
}
